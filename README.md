# 🖍️ Vibrant

License card: <https://stellular.net/+oss/misc.html#vibrant>

Vibrant is the second version of Bundlrs *atomic* pastes. Vibrant can be integrated into Bundlrs through the `VIBRANT_ROOT` environment variable. Vibrant also requires you provide a `BUNDLRS_ROOT` environment variable when running.

## Configuration

Many configuration options for databases can be found [here](https://code.stellular.org/stellular/bundlrs#configuration), this will just detail Vibrant-specific configuration options.

## Containers

Vibrant works by building and serving static files from within a Docker container. To serve these assets from outside the container, we must run a specific Vibrant client within the container that sets up a socket that forwards requests from outside the container to inside. This means we do not use any ports on the host machine.

## File Upload

Files can be uploaded to specific projects that are not of the `StaticContainer` type. These files are served through the assigned project subdomain. Files can be managed using a simple CRUD API:

* `GET /api/v1/project/{PROJECT_NAME}/files` - get all files for the specified `PROJECT_NAME`
* `GET /api/v1/project/{PROJECT_NAME}/files/{PATH}` - read a file
* `POST /api/v1/project/{PROJECT_NAME}/files/{PATH}` - create a file
* `PUT /api/v1/project/{PROJECT_NAME}/files/{PATH}` - update a file
* `DELETE /api/v1/project/{PROJECT_NAME}/files/{PATH}` - delete a file

Files are stored in the database as base64 `BLOB` objects. Files can be at most 2 MB.
