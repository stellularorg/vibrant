use dorsal::query as sqlquery;
use dorsal::DefaultReturn;

use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct AppData {
    pub db: Database,
    pub http_client: awc::Client,
}

// base structures
#[derive(Clone, Serialize, Deserialize)]
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
    pub limit: ProjectRequestLimit,
}

impl Default for ProjectPrivateMetadata {
    fn default() -> Self {
        ProjectPrivateMetadata {
            limit: ProjectRequestLimit::default(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    /// Simple bash script to run deployment commands
    #[serde(default)]
    pub script: String,
}

impl Default for ProjectMetadata {
    fn default() -> Self {
        ProjectMetadata {
            script: String::new(),
        }
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
            full_res.push(PCreateProject {
                name: row.get("name").unwrap().to_string(),
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

        // make sure project does not exist
        let existing = self.get_project_by_id(props.name.clone()).await;

        if existing.success | existing.payload.is_some() {
            return DefaultReturn {
                success: false,
                message: String::from("A project with this name already exists!"),
                payload: Option::None,
            };
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
                .bind::<&String>(&as_user.as_ref().unwrap())
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
            .remove_starting_with(format!("projects-by-owner:{}*", as_user.as_ref().unwrap()))
            .await;

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Project created"),
            payload: Option::Some(props.to_owned()),
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
        let can_edit: bool =
            // using "metadata" here likely prohibits us from changing board owner
            (user.user.username == project.owner) | (user.level.permissions.contains(&String::from("VIB:Admin")));

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
        let can_edit: bool =
            // using "metadata" here likely prohibits us from changing board owner
            (user.user.username == project.owner) | (user.level.permissions.contains(&String::from("VIB:Admin")));

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
}