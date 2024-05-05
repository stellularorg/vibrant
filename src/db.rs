use base64::Engine;
use bollard::secret::HostConfig;
use dorsal::query as sqlquery;
use dorsal::DefaultReturn;

use bollard::container::{Config as ContainerConfig, RemoveContainerOptions};
use bollard::Docker;

use serde::{Deserialize, Serialize};

const KIT_IMAGE: &str = "vibrant_deploy_kit:latest"; // container image

#[derive(Clone)]
pub struct AppData {
    pub db: Database,
    pub http_client: awc::Client,
    pub daemon: Docker,
    pub port: u16,
}

// base structures
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProjectRequestLimit {
    /// project can serve 1,000,000 requests per billing period
    Default = 1_000_000,
    /// project can serve 100,000,000 requests per billing period
    Enterprise = 100_000_000,
    /// project has no request limit (requests SHOULD NOT be tracked if this is set)
    Disabled,
}

impl Default for ProjectRequestLimit {
    fn default() -> Self {
        ProjectRequestLimit::Default
    }
}

impl std::fmt::Display for ProjectRequestLimit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProjectType {
    /// Files are manually uploaded and stored in the database as base64
    StaticPackage,
    /// Files are built inside of a dedicated Docker container/microvm
    StaticContainer,
}

impl Default for ProjectType {
    fn default() -> Self {
        ProjectType::StaticPackage
    }
}

impl std::fmt::Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// base structures
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProjectFilePrivacy {
    /// project files can be LISTED and VIEWED by everybody
    Public,
    /// project files can be LISTED by nobody and VIEWED by everybody
    Confidential,
    /// project files can be LISTED by nobody; files can only be VIEWED by project owner
    Private,
}

impl Default for ProjectFilePrivacy {
    fn default() -> Self {
        ProjectFilePrivacy::Public
    }
}

impl std::fmt::Display for ProjectFilePrivacy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Project {
    /// basically the project ID (no spaces, must be unique)
    pub name: String,
    /// username of the project owner, if this starts with `org:` it will be treated as the name of an [`Organization`]
    pub owner: String,
    /// NOT A CREATION TIMESTAMP, billing period start (limit beginning)
    pub timestamp: u128,
    // metadata
    /// Metadata that can only be edited by users with the "VIB:Admin" permission
    pub private_metadata: ProjectPrivateMetadata,
    /// Public metadata, can be edited by project owner
    pub metadata: ProjectMetadata,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ProjectPrivateMetadata {
    #[serde(default)]
    pub r#type: ProjectType,
    #[serde(default)]
    pub limit: ProjectRequestLimit,
    // container settings
    #[serde(default)]
    pub image: String,
    pub cid: Option<String>,
}

impl Default for ProjectPrivateMetadata {
    fn default() -> Self {
        ProjectPrivateMetadata {
            r#type: ProjectType::default(),
            limit: ProjectRequestLimit::default(),
            // container settings
            image: KIT_IMAGE.to_string(),
            cid: Option::None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    /// Simple bash script to run deployment commands
    #[serde(default)]
    pub script: String,
    #[serde(default)]
    pub file_privacy: ProjectFilePrivacy,
}

impl Default for ProjectMetadata {
    fn default() -> Self {
        ProjectMetadata {
            script: String::new(),
            file_privacy: ProjectFilePrivacy::default(),
        }
    }
}

impl std::fmt::Display for ProjectMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Organization {
    /// must be unique (same requirements as [`Project`] name)
    pub name: String,
    /// username of the organization owner, has permission to delete projects under the organization
    pub owner: String,
    /// this one is a creation timestamp
    pub timestamp: u128,
    /// metadata that can only be edited by the organization owner
    pub metadata: OrganizationMetadata,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OrganizationMetadata {}

impl Default for OrganizationMetadata {
    fn default() -> Self {
        OrganizationMetadata {}
    }
}

// props
#[derive(Clone, Serialize, Deserialize)]
pub struct PCreateProject {
    /// must be unique
    pub name: String,
    pub r#type: ProjectType,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PEditFieldsByName {
    /// must be unique
    pub name: String,
    pub owner: String,
}

// server
#[derive(Clone)]
pub struct Database {
    pub base: dorsal::StarterDatabase,
    pub auth: dorsal::AuthDatabase,
    pub logs: dorsal::LogDatabase,
}

impl Database {
    pub async fn new(opts: dorsal::DatabaseOpts) -> Database {
        let db = dorsal::StarterDatabase::new(opts).await;

        Database {
            base: db.clone(),
            auth: dorsal::AuthDatabase { base: db.clone() },
            logs: dorsal::LogDatabase { base: db },
        }
    }

    pub async fn init(&self) {
        let c = &self.base.db.client;

        let _ = sqlquery(
            "CREATE TABLE IF NOT EXISTS \"Projects\" (
                name VARCHAR(1000000),
                owner VARCHAR(1000000),
                timestamp VARCHAR(1000000),
                private_metadata VARCHAR(1000000),
                metadata VARCHAR(1000000)
            )",
        )
        .execute(c)
        .await;

        let _ = sqlquery(
            "CREATE TABLE IF NOT EXISTS \"ProjectFiles\" (
                project VARCHAR(1000000),
                path VARCHAR(1000000),
                content BLOB
            )",
        )
        .execute(c)
        .await;

        // users and logs tables
        let _ = sqlquery(
            "CREATE TABLE IF NOT EXISTS \"Users\" (
                username VARCHAR(1000000),
                id_hashed VARCHAR(1000000),
                role VARCHAR(1000000),
                timestamp VARCHAR(1000000),
                metadata VARCHAR(1000000)
            )",
        )
        .execute(c)
        .await;

        let _ = sqlquery(
            "CREATE TABLE IF NOT EXISTS \"Logs\" (
                id VARCHAR(1000000),
                logtype VARCHAR(1000000),
                timestamp  VARCHAR(1000000),
                content VARCHAR(1000000)
            )",
        )
        .execute(c)
        .await;
    }

    // projects

    // GET
    /// Get a [`Project`] by its name.
    ///
    /// # Arguments:
    /// * `name` - project name
    pub async fn get_project_by_id(&self, name: String) -> DefaultReturn<Option<Project>> {
        // check in cache
        let cached = self.base.cachedb.get(format!("project:{}", name)).await;

        if cached.is_some() {
            // ...
            let project = serde_json::from_str::<Project>(cached.unwrap().as_str()).unwrap();

            // return
            return DefaultReturn {
                success: true,
                message: String::from("Project exists"),
                payload: Option::Some(project),
            };
        }

        // ...
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "SELECT * FROM \"Projects\" WHERE \"name\" = ?"
        } else {
            "SELECT * FROM \"Projects\" WHERE \"name\" = $1"
        };

        let c = &self.base.db.client;
        let res = sqlquery(query).bind::<&String>(&name).fetch_one(c).await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // ...
        let row = res.unwrap();
        let row = self.base.textify_row(row).data;

        // ...
        // metadata is stored as a string
        let private_metadata =
            serde_json::from_str::<ProjectPrivateMetadata>(row.get("private_metadata").unwrap())
                .unwrap();

        let metadata =
            serde_json::from_str::<ProjectMetadata>(row.get("metadata").unwrap()).unwrap();

        let project = Project {
            name: row.get("name").unwrap().to_string(),
            owner: row.get("owner").unwrap().to_string(),
            timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
            private_metadata,
            metadata,
        };

        // store in cache
        self.base
            .cachedb
            .set(
                format!("project:{}", name),
                serde_json::to_string::<Project>(&project).unwrap(),
            )
            .await;

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Project exists"),
            payload: Option::Some(project),
        };
    }

    /// Get all [projects](PCreateProject) owned by a specific user (limited), sorted by billing period start
    ///
    /// # Arguments:
    /// * `owner` - `String` of the owner's `username`
    /// * `offset` - optional value representing the SQL fetch offset
    pub async fn get_projects_by_owner_limited(
        &self,
        owner: String,
        offset: Option<i32>,
    ) -> DefaultReturn<Option<Vec<PCreateProject>>> {
        let offset = if offset.is_some() { offset.unwrap() } else { 0 };

        // check in cache
        let cached = self
            .base
            .cachedb
            .get(format!("projects-by-owner:{}:offset{}", owner, offset))
            .await;

        if cached.is_some() {
            // ...
            let projects =
                serde_json::from_str::<Vec<PCreateProject>>(cached.unwrap().as_str()).unwrap();

            // return
            return DefaultReturn {
                success: true,
                message: owner,
                payload: Option::Some(projects),
            };
        }

        // ...
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "SELECT * FROM \"Projects\" WHERE \"owner\" = ? ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET ?"
        } else {
            "SELECT * FROM \"Projects\" WHERE \"owner\" = $1 ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET $2"
        };

        let c = &self.base.db.client;
        let res = sqlquery(query)
            .bind::<&String>(&owner)
            .bind(offset)
            .fetch_all(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // build res
        let mut full_res: Vec<PCreateProject> = Vec::new();

        for row in res.unwrap() {
            let row = self.base.textify_row(row).data;
            let metadata = serde_json::from_str::<ProjectPrivateMetadata>(
                row.get("private_metadata").unwrap(),
            )
            .unwrap();

            full_res.push(PCreateProject {
                name: row.get("name").unwrap().to_string(),
                r#type: metadata.r#type,
            });
        }

        // store in cache
        self.base
            .cachedb
            .set(
                format!("projects-by-owner:{}:offset{}", owner, offset),
                serde_json::to_string::<Vec<PCreateProject>>(&full_res).unwrap(),
            )
            .await;

        // return
        return DefaultReturn {
            success: true,
            message: owner,
            payload: Option::Some(full_res),
        };
    }

    // SET
    /// Create a new [`Project`] given various [`properties`](PCreateProject)
    ///
    /// # Arguments:
    /// * `props` - [`(PROPS)CreateProject`](PCreateProject)
    /// * `as_user` - The username of the user creating the project
    pub async fn create_project(
        &self,
        props: &mut PCreateProject,
        as_user: Option<String>, // username of owner
        daemon: Docker,
        port: u16,
    ) -> DefaultReturn<Option<PCreateProject>> {
        // make sure we're authenticated
        if as_user.is_none() {
            return DefaultReturn {
                success: false,
                message: String::from("You must have an account to do this."),
                payload: Option::None,
            };
        }

        // check values

        // (check length)
        if (props.name.len() < 2) | (props.name.len() > 500) {
            return DefaultReturn {
                success: false,
                message: String::from("Name is invalid"),
                payload: Option::None,
            };
        }

        // (characters used)
        let regex = regex::RegexBuilder::new("^[\\w\\_\\-]+$")
            .multi_line(true)
            .build()
            .unwrap();

        if regex.captures(&props.name).iter().len() < 1 {
            return DefaultReturn {
                success: false,
                message: String::from("Name is invalid"),
                payload: Option::None,
            };
        }

        // project cannot have names we may need
        if ["dashboard", "api"].contains(&props.name.as_str()) {
            return DefaultReturn {
                success: false,
                message: String::from("Name is invalid"),
                payload: Option::None,
            };
        }

        // make sure project does not exist
        let existing = self.get_project_by_id(props.name.clone()).await;

        if existing.success | existing.payload.is_some() {
            return DefaultReturn {
                success: false,
                message: String::from("A project with this name already exists!"),
                payload: Option::None,
            };
        }

        // get user
        let user = self.auth.get_user_by_username(as_user.unwrap()).await;

        if !user.success {
            return DefaultReturn {
                success: false,
                message: String::from("User is invalid!"),
                payload: Option::None,
            };
        }

        let user = user.payload.unwrap();

        // get user projects for count
        if !user
            .level
            .permissions
            .contains(&"VIB:MaxProjects:Disabled".to_string())
        {
            let mut max_of_10 = user
                .level
                .permissions
                .contains(&"VIB:MaxProjects:10".to_string());
            let max_of_25 = user
                .level
                .permissions
                .contains(&"VIB:MaxProjects:25".to_string());

            // if both are false, max_of_10 should be true
            if (max_of_10 == false) && (max_of_25 == false) {
                max_of_10 = true;
            }

            // ...
            let user_projects = self
                .get_projects_by_owner_limited(user.user.username.clone(), Option::Some(0))
                .await;

            if !user_projects.success {
                return DefaultReturn {
                    success: false,
                    message: String::from("User is invalid!"),
                    payload: Option::None,
                };
            }

            let user_projects = user_projects.payload.unwrap();

            // ...
            if max_of_10 && user_projects.len() >= 10 {
                return DefaultReturn {
                    success: false,
                    message: String::from("You have reached the maximum number of projects allowed for your account level."),
                    payload: Option::None,
                };
            } else if max_of_25 && user_projects.len() >= 25 {
                return DefaultReturn {
                    success: false,
                    message: String::from("You have reached the maximum number of projects allowed for your account level."),
                    payload: Option::None,
                };
            }
        }

        // create project
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "INSERT INTO \"Projects\" VALUES (?, ?, ?, ?, ?)"
        } else {
            "INSERT INTO \"Projects\" VALUES ($1, $2, $3, $4, $5)"
        };

        let c = &self.base.db.client;
        let res =
            sqlquery(query)
                .bind::<&String>(&props.name)
                .bind::<&String>(&user.user.username)
                .bind::<&String>(&dorsal::utility::unix_epoch_timestamp().to_string()) // billing period starts now
                .bind::<&String>(
                    &serde_json::to_string::<ProjectPrivateMetadata>(
                        &ProjectPrivateMetadata::default(),
                    )
                    .unwrap(),
                )
                .bind::<&String>(
                    &serde_json::to_string::<ProjectMetadata>(&ProjectMetadata::default()).unwrap(),
                )
                .execute(c)
                .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: res.err().unwrap().to_string(),
                payload: Option::None,
            };
        }

        // clear user projects at all layers
        self.base
            .cachedb
            .remove_starting_with(format!("projects-by-owner:{}:*", user.user.username))
            .await;

        // create container
        self.create_project_container(props.name.clone(), daemon, port)
            .await;

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Project created"),
            payload: Option::Some(props.to_owned()),
        };
    }

    /// Update a [`Project`]'s [`fields`](PEditFieldsByName) by its `name`
    pub async fn edit_fields_by_name(
        &self,
        name: String,
        mut fields: PEditFieldsByName,
        edit_as: Option<String>, // username of account that is editing this project
    ) -> DefaultReturn<Option<String>> {
        // make sure project exists
        let existing = &self.get_project_by_id(name.clone()).await;
        if !existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("Project does not exist!"),
                payload: Option::None,
            };
        }

        let project = existing.payload.as_ref().unwrap();

        // make sure the new name is valid
        if fields.name.len() < 2 {
            fields.name = project.name.clone();
        }

        // get edit_as user account
        let ua = if edit_as.is_some() {
            Option::Some(
                self.auth
                    .get_user_by_username(edit_as.clone().unwrap())
                    .await
                    .payload,
            )
        } else {
            Option::None
        };

        if ua.is_none() {
            return DefaultReturn {
                success: false,
                message: String::from("An account is required to do this"),
                payload: Option::None,
            };
        }

        // make sure we can do this
        let user = ua.unwrap().unwrap();
        let can_edit: bool = (user.user.username == project.owner)
            | (user.level.permissions.contains(&String::from("VIB:Admin")));

        if can_edit == false {
            return DefaultReturn {
                success: false,
                message: String::from(
                    "You do not have permission to manage this project's contents.",
                ),
                payload: Option::None,
            };
        }

        // if user does not have correct permission to edit owner
        if !user
            .level
            .permissions
            .contains(&"VIB:Actions:EditOwner".to_string())
        {
            fields.owner = user.user.username.clone();
        }

        // check if project already exists under new name
        if name != fields.name {
            let existing = &self.get_project_by_id(fields.name.clone()).await;

            if existing.success {
                return DefaultReturn {
                    success: false,
                    message: String::from("This project name is already in use!"),
                    payload: Option::None,
                };
            }
        }

        // update project
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "UPDATE \"Projects\" SET \"owner\" = ?, \"name\" = ? WHERE \"name\" = ?"
        } else {
            "UPDATE \"Projects\" SET (\"owner\", \"name\") = ($1, $2) WHERE \"name\" = $2"
        };

        let c = &self.base.db.client;
        let res = sqlquery(query)
            .bind::<&String>(&fields.owner)
            .bind::<&String>(&fields.name)
            .bind::<&String>(&name)
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // update files
        if name != fields.name {
            let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql")
            {
                "UPDATE \"ProjectFiles\" SET \"project\" = ? WHERE \"project\" = ?"
            } else {
                "UPDATE \"ProjectFiles\" SET (\"project\") = ($1) WHERE \"project\" = $2"
            };

            let c = &self.base.db.client;
            let res = sqlquery(query)
                .bind::<&String>(&fields.name)
                .bind::<&String>(&name)
                .execute(c)
                .await;

            if res.is_err() {
                return DefaultReturn {
                    success: false,
                    message: String::from(res.err().unwrap().to_string()),
                    payload: Option::None,
                };
            }
        }

        // update cache
        let existing_in_cache = self.base.cachedb.get(format!("project:{}", name)).await;

        if existing_in_cache.is_some() {
            let mut project = serde_json::from_str::<Project>(&existing_in_cache.unwrap()).unwrap();

            project.owner = fields.owner;
            project.name = fields.name.clone();

            if name != fields.name {
                // remove old
                self.base.cachedb.remove(format!("project:{}", name)).await;

                self.base
                    .cachedb
                    .remove_starting_with(format!("project:{}:*", name))
                    .await;

                self.base
                    .cachedb
                    .remove_starting_with(format!("projects-by-owner:{}:*", user.user.username))
                    .await;

                // insert new
                self.base
                    .cachedb
                    .set(
                        format!("project:{}", fields.name),
                        serde_json::to_string::<Project>(&project).unwrap(),
                    )
                    .await;
            } else {
                // update cache
                self.base
                    .cachedb
                    .update(
                        format!("project:{}", name),
                        serde_json::to_string::<Project>(&project).unwrap(),
                    )
                    .await;
            }
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Project updated!"),
            payload: Option::Some(fields.name),
        };
    }

    /// Increment a [`Project`]'s billing request limit (redis)
    ///
    /// # Arguments:
    /// * `name` - project name
    pub async fn incr_project_requests(&self, name: String) -> bool {
        // make sure project exists
        let existing = self.get_project_by_id(name.clone()).await;

        if !existing.success | existing.payload.is_none() {
            return false;
        }

        let project = existing.payload.unwrap();

        // check project timestamp
        let now = dorsal::utility::unix_epoch_timestamp();

        // if it's been a month or more, reset billing timestamp
        if (now - project.timestamp) >= 2629800000 {
            // initiate bill (TODO)
            // self.bill_project(name).await;
            let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql")
            {
                "UPDATE \"Projects\" SET (\"timestamp\") = (?)"
            } else {
                "UPDATE \"Projects\" SET (\"timestamp\") = ($1)"
            };

            let c = &self.base.db.client;
            let res = sqlquery(query)
                .bind::<&String>(&now.to_string())
                .execute(c)
                .await;

            if res.is_err() {
                return false;
            }
        }

        // incr requests
        self.base
            .cachedb
            .incr(format!("billing:requests:{}", name))
            .await;

        // return
        true
    }

    /// Update a [`Project`]'s [`metadata`](ProjectMetadata) by its `name`
    pub async fn edit_project_metadata_by_name(
        &self,
        name: String,
        metadata: ProjectMetadata,
        edit_as: Option<String>, // username of account that is editing this project
    ) -> DefaultReturn<Option<String>> {
        // make sure project exists
        let existing = &self.get_project_by_id(name.clone()).await;
        if !existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("Project does not exist!"),
                payload: Option::None,
            };
        }

        let project = existing.payload.as_ref().unwrap();

        // get edit_as user account
        let ua = if edit_as.is_some() {
            Option::Some(
                self.auth
                    .get_user_by_username(edit_as.clone().unwrap())
                    .await
                    .payload,
            )
        } else {
            Option::None
        };

        if ua.is_none() {
            return DefaultReturn {
                success: false,
                message: String::from("An account is required to do this"),
                payload: Option::None,
            };
        }

        // make sure we can do this
        let user = ua.unwrap().unwrap();
        let can_edit: bool = (user.user.username == project.owner)
            | (user.level.permissions.contains(&String::from("VIB:Admin")));

        if can_edit == false {
            return DefaultReturn {
                success: false,
                message: String::from(
                    "You do not have permission to manage this project's contents.",
                ),
                payload: Option::None,
            };
        }

        // update project
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "UPDATE \"Projects\" SET \"metadata\" = ? WHERE \"name\" = ?"
        } else {
            "UPDATE \"Projects\" SET (\"metadata\") = ($1) WHERE \"name\" = $2"
        };

        let c = &self.base.db.client;
        let metadata_string = serde_json::to_string(&metadata).unwrap();
        let res = sqlquery(query)
            .bind::<&String>(&metadata_string)
            .bind::<&String>(&name)
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // update cache
        let existing_in_cache = self.base.cachedb.get(format!("project:{}", name)).await;

        if existing_in_cache.is_some() {
            let mut project = serde_json::from_str::<Project>(&existing_in_cache.unwrap()).unwrap();
            project.metadata = metadata; // update metadata

            // update cache
            self.base
                .cachedb
                .update(
                    format!("project:{}", name),
                    serde_json::to_string::<Project>(&project).unwrap(),
                )
                .await;
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Project updated!"),
            payload: Option::Some(name),
        };
    }

    /// Update a [`Project`]'s [`private metadata`](ProjectPrivateMetadata) by its `name`
    pub async fn edit_project_private_metadata_by_name(
        &self,
        name: String,
        metadata: ProjectPrivateMetadata,
        edit_as: Option<String>, // username of account that is editing this project
    ) -> DefaultReturn<Option<String>> {
        // make sure project exists
        let existing = &self.get_project_by_id(name.clone()).await;
        if !existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("Project does not exist!"),
                payload: Option::None,
            };
        }

        let project = existing.payload.as_ref().unwrap();

        // get edit_as user account
        let ua = if edit_as.is_some() {
            Option::Some(
                self.auth
                    .get_user_by_username(edit_as.clone().unwrap())
                    .await
                    .payload,
            )
        } else {
            Option::None
        };

        // if ua.is_none() {
        //     return DefaultReturn {
        //         success: false,
        //         message: String::from("An account is required to do this"),
        //         payload: Option::None,
        //     };
        // }

        // make sure we can do this
        if ua.is_some() {
            let user = ua.unwrap().unwrap();
            let can_edit: bool = (user.user.username == project.owner)
                | (user.level.permissions.contains(&String::from("VIB:Admin")));

            if can_edit == false {
                return DefaultReturn {
                    success: false,
                    message: String::from(
                        "You do not have permission to manage this project's contents.",
                    ),
                    payload: Option::None,
                };
            }
        }

        // update project
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "UPDATE \"Projects\" SET \"private_metadata\" = ? WHERE \"name\" = ?"
        } else {
            "UPDATE \"Projects\" SET (\"private_metadata\") = ($1) WHERE \"name\" = $2"
        };

        let c = &self.base.db.client;
        let metadata_string = serde_json::to_string(&metadata).unwrap();
        let res = sqlquery(query)
            .bind::<&String>(&metadata_string)
            .bind::<&String>(&name)
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // update cache
        let existing_in_cache = self.base.cachedb.get(format!("project:{}", name)).await;

        if existing_in_cache.is_some() {
            let mut project = serde_json::from_str::<Project>(&existing_in_cache.unwrap()).unwrap();
            project.private_metadata = metadata; // update metadata

            // update cache
            self.base
                .cachedb
                .update(
                    format!("project:{}", name),
                    serde_json::to_string::<Project>(&project).unwrap(),
                )
                .await;
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Project updated!"),
            payload: Option::Some(name),
        };
    }

    /// Delete a [`Project`] given its `name`
    pub async fn delete_project(
        &self,
        name: String,
        delete_as: Option<String>, // username of account that is deleting this project
        daemon: Docker,
    ) -> DefaultReturn<Option<String>> {
        // make sure project exists
        let existing = &self.get_project_by_id(name.clone()).await;
        if !existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("Project does not exist!"),
                payload: Option::None,
            };
        }

        let project = existing.payload.as_ref().unwrap();

        // get delete_as user account
        let ua = if delete_as.is_some() {
            Option::Some(
                self.auth
                    .get_user_by_username(delete_as.clone().unwrap())
                    .await
                    .payload,
            )
        } else {
            Option::None
        };

        // make sure we can do this
        if ua.is_some() {
            let user = ua.unwrap().unwrap();
            let can_delete: bool = (user.user.username == project.owner)
                | (user.level.permissions.contains(&String::from("VIB:Admin")));

            if can_delete == false {
                return DefaultReturn {
                    success: false,
                    message: String::from(
                        "You do not have permission to manage this project's contents.",
                    ),
                    payload: Option::None,
                };
            }
        }

        // update project
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "DELETE FROM \"Projects\" WHERE \"name\" = ?"
        } else {
            "DELETE FROM \"Projects\" WHERE \"name\" = $2"
        };

        let c = &self.base.db.client;
        let res = sqlquery(query).bind::<&String>(&name).execute(c).await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // remove container
        self.remove_project_container(name.clone(), daemon).await;

        // remove files
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "DELETE FROM \"ProjectFiles\" WHERE \"project\" = ?"
        } else {
            "DELETE FROM \"ProjectFiles\" WHERE \"project\" = $2"
        };

        let c = &self.base.db.client;
        let res = sqlquery(query).bind::<&String>(&name).execute(c).await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // update cache
        // self.base.cachedb.remove(format!("project:{}", name)).await;
        self.base.cachedb.remove(format!("project:{}", name)).await;
        self.base
            .cachedb
            .remove_starting_with(format!("project:{}:*", name))
            .await;

        if delete_as.is_some() {
            self.base
                .cachedb
                .remove_starting_with(format!(
                    "projects-by-owner:{}:*",
                    delete_as.as_ref().unwrap()
                ))
                .await;
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Project deleted!"),
            payload: Option::Some(name),
        };
    }

    // files

    // GET
    /// Get a file by `path` in the given [`Project`]
    pub async fn get_file_in_project(
        &self,
        name: String,
        mut path: String,
        as_user: Option<String>,
        bypass_user_checks: bool,
    ) -> DefaultReturn<Option<Vec<u8>>> {
        // get project
        let existing = self.get_project_by_id(name.clone()).await;

        if existing.success == false {
            return DefaultReturn {
                success: false,
                message: String::from("Project does not exist!"),
                payload: Option::None,
            };
        }

        let project = existing.payload.unwrap();

        // check file privacy
        if bypass_user_checks == false {
            if as_user.is_some() {
                let user = as_user.unwrap();

                // "Confidential" is basically the same as "Public" in ProjectFilePrivacy
                if (project.metadata.file_privacy == ProjectFilePrivacy::Private)
                    && (user != project.owner)
                {
                    return DefaultReturn {
                        success: false,
                        message: String::from("Not allowed to view project files!"),
                        payload: Option::None,
                    };
                }
            } else {
                // TODO: possibly make "Public" be required here (make "Confidential" hide from non-authenticated users)
                if project.metadata.file_privacy == ProjectFilePrivacy::Private {
                    return DefaultReturn {
                        success: false,
                        message: String::from("Not allowed to view project files!"),
                        payload: Option::None,
                    };
                }
            }
        }

        // check path
        if path == "/" {
            path = String::from("/index.html");
        } else if !path.starts_with("/") {
            path = format!("/{}", path);
        }

        // get project owner
        let user = self.auth.get_user_by_username(project.owner).await;

        if user.success == false {
            return DefaultReturn {
                success: false,
                message: String::from("Project owner is invalid!"),
                payload: Option::None,
            };
        }

        // check permission
        let user = user.payload.unwrap();

        if !user
            .level
            .permissions
            .contains(&"VIB:RequestLimit:Disabled".to_string())
        {
            let user_limit = if user
                .level
                .permissions
                .contains(&"VIB:RequestLimit:Enterprise".to_string())
            {
                ProjectRequestLimit::Enterprise
            } else {
                ProjectRequestLimit::Default
            };

            let current_usage = self
                .base
                .cachedb
                .get(format!("billing:requests:{}", name))
                .await
                .unwrap_or(String::from("0"))
                .parse::<i32>()
                .unwrap();

            // ...
            if ((user_limit == ProjectRequestLimit::Enterprise) && (current_usage >= 100_000_000))
                | ((user_limit == ProjectRequestLimit::Default) && (current_usage >= 1_000_000))
            {
                return DefaultReturn {
                    success: false,
                    message: String::from("Limit exceeded!"),
                    payload: Option::None,
                };
            }
        }

        // check in cache
        let cached = self
            .base
            .cachedb
            .get(format!("project:{}:path:{}", name, path))
            .await;

        if cached.is_some() {
            // ...
            let content = cached.unwrap();

            // decode
            let bytes = base64::engine::general_purpose::STANDARD.decode(content);

            if bytes.is_err() {
                return DefaultReturn {
                    success: false,
                    message: String::from(bytes.err().unwrap().to_string()),
                    payload: Option::None,
                };
            }

            let bytes = bytes.unwrap();

            // return
            return DefaultReturn {
                success: true,
                message: path,
                payload: Option::Some(bytes),
            };
        }

        // ...
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "SELECT * FROM \"ProjectFiles\" WHERE \"project\" = ? AND \"path\" = ?"
        } else {
            "SELECT * FROM \"ProjectFiles\" WHERE \"project\" = $1 AND \"path\" = $2"
        };

        let c = &self.base.db.client;
        let res = sqlquery(query)
            .bind::<&String>(&name)
            .bind::<&String>(&path)
            .fetch_one(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Unable to find file at given path!"),
                payload: Option::None,
            };
        }

        // ...
        let row = res.unwrap();
        let row = self.base.textify_row(row).data;

        // ...
        let original_base64 = row.get("content").unwrap();
        let bytes = base64::engine::general_purpose::STANDARD.decode(original_base64);

        if bytes.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(bytes.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        let bytes = bytes.unwrap();

        // store in cache
        self.base
            .cachedb
            .set(
                format!("project:{}:path:{}", name, path),
                original_base64.to_string(),
            )
            .await;

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Path exists"),
            payload: Option::Some(bytes),
        };
    }

    /// Get all file (names) in the given [`Project`]
    pub async fn get_project_files(
        &self,
        name: String,
        as_user: Option<String>,
        bypass_user_checks: bool,
    ) -> DefaultReturn<Vec<String>> {
        // get project
        let existing = self.get_project_by_id(name.clone()).await;

        if existing.success == false {
            return DefaultReturn {
                success: false,
                message: String::from("Project does not exist!"),
                payload: Vec::new(),
            };
        }

        // check permissions
        if bypass_user_checks == false {
            let project = existing.payload.unwrap();

            if as_user.is_some() {
                let user = as_user.unwrap();

                if (project.metadata.file_privacy != ProjectFilePrivacy::Public)
                    && (user != project.owner)
                {
                    return DefaultReturn {
                        success: false,
                        message: String::from("Not allowed to view project file listing!"),
                        payload: Vec::new(),
                    };
                }
            } else {
                if project.metadata.file_privacy != ProjectFilePrivacy::Public {
                    return DefaultReturn {
                        success: false,
                        message: String::from("Not allowed to view project file listing!"),
                        payload: Vec::new(),
                    };
                }
            }
        }

        // incr project requests
        // self.incr_project_requests(name.clone()).await;

        // check in cache
        // let cached = self
        //     .base
        //     .cachedb
        //     .get(format!("project:{}:path:{}", name, path))
        //     .await;

        // if cached.is_some() {
        //     // ...
        //     let content = cached.unwrap();

        //     // decode
        //     let bytes = base64::engine::general_purpose::STANDARD.decode(content);

        //     if bytes.is_err() {
        //         return DefaultReturn {
        //             success: false,
        //             message: String::from(bytes.err().unwrap().to_string()),
        //             payload: Option::None,
        //         };
        //     }

        //     let bytes = bytes.unwrap();

        //     // return
        //     return DefaultReturn {
        //         success: true,
        //         message: path,
        //         payload: Option::Some(bytes),
        //     };
        // }

        // ...
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "SELECT \"path\" FROM \"ProjectFiles\" WHERE \"project\" = ?"
        } else {
            "SELECT \"path\" FROM \"ProjectFiles\" WHERE \"project\" = $1"
        };

        let c = &self.base.db.client;
        let res = sqlquery(query).bind::<&String>(&name).fetch_all(c).await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Vec::new(),
            };
        }

        // ...
        // build res
        let mut full_res: Vec<String> = Vec::new();

        for row in res.unwrap() {
            let row = self.base.textify_row(row).data;
            full_res.push(row.get("path").unwrap().to_string());
        }

        // store in cache
        // self.base
        //     .cachedb
        //     .set(
        //         format!("project:{}:files", name),
        //         original_base64.to_string(),
        //     )
        //     .await;

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Files exist"),
            payload: full_res,
        };
    }

    // SET
    /// Create a file by `path` in the given [`Project`]
    pub async fn store_file_in_project(
        &self,
        name: String,
        mut path: String,
        content: String, // base64 content
        edit_as: Option<String>,
    ) -> DefaultReturn<Option<String>> {
        // get project
        let existing = self.get_project_by_id(name.clone()).await;

        if existing.success == false {
            return DefaultReturn {
                success: false,
                message: String::from("Project does not exist!"),
                payload: Option::None,
            };
        }

        let project = existing.payload.unwrap();

        // get edit_as user account
        let ua = if edit_as.is_some() {
            Option::Some(
                self.auth
                    .get_user_by_username(edit_as.clone().unwrap())
                    .await
                    .payload,
            )
        } else {
            Option::None
        };

        if ua.is_none() {
            return DefaultReturn {
                success: false,
                message: String::from("An account is required to do this"),
                payload: Option::None,
            };
        }

        // make sure we can do this
        let user = ua.unwrap().unwrap();
        let can_edit: bool = (user.user.username == project.owner)
            | (user.level.permissions.contains(&String::from("VIB:Admin")));

        if can_edit == false {
            return DefaultReturn {
                success: false,
                message: String::from(
                    "You do not have permission to manage this project's contents.",
                ),
                payload: Option::None,
            };
        }

        // check path
        if !path.starts_with("/") {
            path = format!("/{}", path);
        }

        // ...
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "INSERT INTO \"ProjectFiles\" VALUES (?, ?, ?)"
        } else {
            "INSERT INTO \"ProjectFiles\" VALUES ($1, $2, $3)"
        };

        let c = &self.base.db.client;
        let res = sqlquery(query)
            .bind::<&String>(&name)
            .bind::<&String>(&path)
            .bind::<&String>(&content)
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // store in cache
        self.base
            .cachedb
            .set(format!("project:{}:path:{}", name, path), content)
            .await;

        // return
        return DefaultReturn {
            success: true,
            message: String::from("File inserted"),
            payload: Option::Some(path),
        };
    }

    /// Update a file by `path` in the given [`Project`]
    pub async fn update_file_in_project(
        &self,
        name: String,
        mut path: String,
        content: String, // base64 content
        edit_as: Option<String>,
    ) -> DefaultReturn<Option<String>> {
        // get project
        let existing = self.get_project_by_id(name.clone()).await;

        if existing.success == false {
            return DefaultReturn {
                success: false,
                message: String::from("Project does not exist!"),
                payload: Option::None,
            };
        }

        let project = existing.payload.unwrap();

        // get edit_as user account
        let ua = if edit_as.is_some() {
            Option::Some(
                self.auth
                    .get_user_by_username(edit_as.clone().unwrap())
                    .await
                    .payload,
            )
        } else {
            Option::None
        };

        if ua.is_none() {
            return DefaultReturn {
                success: false,
                message: String::from("An account is required to do this"),
                payload: Option::None,
            };
        }

        // make sure we can do this
        let user = ua.unwrap().unwrap();
        let can_edit: bool = (user.user.username == project.owner)
            | (user.level.permissions.contains(&String::from("VIB:Admin")));

        if can_edit == false {
            return DefaultReturn {
                success: false,
                message: String::from(
                    "You do not have permission to manage this project's contents.",
                ),
                payload: Option::None,
            };
        }

        // check path
        if !path.starts_with("/") {
            path = format!("/{}", path);
        }

        // ...
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "UPDATE \"ProjectFiles\" SET \"content\" = ? WHERE \"project\" = ? AND \"path\" = ?"
        } else {
            "UPDATE \"ProjectFiles\" SET (\"content\") = ($1) WHERE \"project\" = $2 AND \"path\" = $3"
        };

        let c = &self.base.db.client;
        let res = sqlquery(query)
            .bind::<&String>(&content)
            .bind::<&String>(&name)
            .bind::<&String>(&path)
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // store in cache
        self.base
            .cachedb
            .set(format!("project:{}:path:{}", name, path), content)
            .await;

        // return
        return DefaultReturn {
            success: true,
            message: String::from("File updated"),
            payload: Option::Some(path),
        };
    }

    /// Delete a file by `path` in the given [`Project`]
    pub async fn delete_file_in_project(
        &self,
        name: String,
        path: String,
        edit_as: Option<String>,
    ) -> DefaultReturn<Option<String>> {
        // get project
        let existing = self.get_project_by_id(name.clone()).await;

        if existing.success == false {
            return DefaultReturn {
                success: false,
                message: String::from("Project does not exist!"),
                payload: Option::None,
            };
        }

        let project = existing.payload.unwrap();

        // get edit_as user account
        let ua = if edit_as.is_some() {
            Option::Some(
                self.auth
                    .get_user_by_username(edit_as.clone().unwrap())
                    .await
                    .payload,
            )
        } else {
            Option::None
        };

        if ua.is_none() {
            return DefaultReturn {
                success: false,
                message: String::from("An account is required to do this"),
                payload: Option::None,
            };
        }

        // make sure we can do this
        let user = ua.unwrap().unwrap();
        let can_edit: bool = (user.user.username == project.owner)
            | (user.level.permissions.contains(&String::from("VIB:Admin")));

        if can_edit == false {
            return DefaultReturn {
                success: false,
                message: String::from(
                    "You do not have permission to manage this project's contents.",
                ),
                payload: Option::None,
            };
        }

        // ...
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "DELETE FROM \"ProjectFiles\" WHERE \"project\" = ? AND \"path\" = ?"
        } else {
            "DELETE FROM \"ProjectFiles\" WHERE \"project\" = $1 AND \"path\" = $2"
        };

        let c = &self.base.db.client;
        let res = sqlquery(query)
            .bind::<&String>(&name)
            .bind::<&String>(&path)
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // remove from cache
        self.base
            .cachedb
            .remove(format!("project:{}:path:{}", name, path))
            .await;

        // return
        return DefaultReturn {
            success: true,
            message: String::from("File deleted"),
            payload: Option::Some(path),
        };
    }

    /// Move a file by `path` to `new_path` in the given [`Project`]
    pub async fn move_file_in_project(
        &self,
        name: String,
        mut path: String,
        mut new_path: String,
        edit_as: Option<String>,
    ) -> DefaultReturn<Option<String>> {
        // get project
        let existing = self.get_project_by_id(name.clone()).await;

        if existing.success == false {
            return DefaultReturn {
                success: false,
                message: String::from("Project does not exist!"),
                payload: Option::None,
            };
        }

        let project = existing.payload.unwrap();

        // get edit_as user account
        let ua = if edit_as.is_some() {
            Option::Some(
                self.auth
                    .get_user_by_username(edit_as.clone().unwrap())
                    .await
                    .payload,
            )
        } else {
            Option::None
        };

        if ua.is_none() {
            return DefaultReturn {
                success: false,
                message: String::from("An account is required to do this"),
                payload: Option::None,
            };
        }

        // make sure we can do this
        let user = ua.unwrap().unwrap();
        let can_edit: bool = (user.user.username == project.owner)
            | (user.level.permissions.contains(&String::from("VIB:Admin")));

        if can_edit == false {
            return DefaultReturn {
                success: false,
                message: String::from(
                    "You do not have permission to manage this project's contents.",
                ),
                payload: Option::None,
            };
        }

        // check path
        if !path.starts_with("/") {
            path = format!("/{}", path);
        }

        if !new_path.starts_with("/") {
            new_path = format!("/{}", new_path);
        }

        // ...
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "UPDATE \"ProjectFiles\" SET \"path\" = ? WHERE \"project\" = ? AND \"path\" = ?"
        } else {
            "UPDATE \"ProjectFiles\" SET (\"path\") = ($1) WHERE \"project\" = $2 AND \"path\" = $3"
        };

        let c = &self.base.db.client;
        let res = sqlquery(query)
            .bind::<&String>(&new_path)
            .bind::<&String>(&name)
            .bind::<&String>(&path)
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // remove from cache
        self.base
            .cachedb
            .remove(format!("project:{}:path:{}", name, path))
            .await;

        // return
        return DefaultReturn {
            success: true,
            message: String::from("File moved"),
            payload: Option::Some(path),
        };
    }

    // docker

    // SET

    /// Create a Docker container for a given [`Project`]
    pub async fn create_project_container(
        &self,
        name: String,
        daemon: Docker,
        port: u16,
    ) -> DefaultReturn<Option<String>> {
        // get project
        let existing = self.get_project_by_id(name.clone()).await;

        if existing.success == false {
            return DefaultReturn {
                success: false,
                message: String::from("Project does not exist!"),
                payload: Option::None,
            };
        }

        // create container
        let mut metadata = existing.payload.unwrap().private_metadata;

        if metadata.r#type != ProjectType::StaticContainer {
            return DefaultReturn {
                success: true,
                message: String::from("Project is not a container project"),
                payload: Option::None,
            };
        }

        let res = daemon
            .create_container::<&str, &str>(
                Option::None,
                ContainerConfig {
                    image: Option::Some(&metadata.image),
                    env: Option::Some(vec![
                        &format!("VIBRANT_URL=\"http://localhost:{port}\""),
                        &format!("PROJECT=\"{name}\""),
                    ]),
                    host_config: Option::Some(HostConfig {
                        extra_hosts: Option::Some(vec![
                            // add host network so we can connect to vibrant
                            "host.docker.internal:host-gateway".to_string(),
                        ]),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to create container"),
                payload: Option::None,
            };
        }

        let cid = res.unwrap().id;
        metadata.cid = Option::Some(cid.clone());

        // store cid
        self.edit_project_private_metadata_by_name(name, metadata, Option::None)
            .await;

        // default return
        return DefaultReturn {
            success: false,
            message: String::from("Started container"),
            payload: Option::Some(cid),
        };
    }

    /// Start a [`Project`] container based on its stored CID
    pub async fn start_project_container(
        &self,
        name: String,
        daemon: Docker,
    ) -> DefaultReturn<Option<String>> {
        // get project
        let existing = self.get_project_by_id(name.clone()).await;

        if existing.success == false {
            return DefaultReturn {
                success: false,
                message: String::from("Project does not exist!"),
                payload: Option::None,
            };
        }

        // get cid
        let metadata = existing.payload.unwrap().private_metadata;

        if metadata.r#type != ProjectType::StaticContainer {
            return DefaultReturn {
                success: true,
                message: String::from("Project is not a container project"),
                payload: Option::None,
            };
        }

        let cid = metadata.cid;

        if cid.is_none() {
            return DefaultReturn {
                success: false,
                message: String::from("Project container does not exist"),
                payload: Option::None,
            };
        }

        // start container
        let res = daemon
            .start_container::<String>(&cid.as_ref().unwrap(), Option::None)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to start project container"),
                payload: Option::None,
            };
        }

        // default return
        return DefaultReturn {
            success: false,
            message: String::from("Started container"),
            payload: Option::Some(cid.unwrap()),
        };
    }

    /// Remove a [`Project`] container based on its stored CID
    pub async fn remove_project_container(
        &self,
        name: String,
        daemon: Docker,
    ) -> DefaultReturn<Option<String>> {
        // get project
        let existing = self.get_project_by_id(name.clone()).await;

        if existing.success == false {
            return DefaultReturn {
                success: false,
                message: String::from("Project does not exist!"),
                payload: Option::None,
            };
        }

        // get cid
        let metadata = existing.payload.unwrap().private_metadata;

        if metadata.r#type != ProjectType::StaticContainer {
            return DefaultReturn {
                success: true,
                message: String::from("Project is not a container project"),
                payload: Option::None,
            };
        }

        let cid = metadata.cid;

        if cid.is_none() {
            return DefaultReturn {
                success: false,
                message: String::from("Project container does not exist"),
                payload: Option::None,
            };
        }

        // delete container
        let res = daemon
            .remove_container(
                &cid.as_ref().unwrap(),
                Option::Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to delete project container"),
                payload: Option::None,
            };
        }

        // default return
        return DefaultReturn {
            success: false,
            message: String::from("Container deleted"),
            payload: Option::Some(cid.unwrap()),
        };
    }
}
