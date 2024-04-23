# 🖍️ Vibrant

License card: <https://stellular.net/+oss/misc.html#vibrant>

## Configuration

Many configuration options for databases can be found [here](https://code.stellular.org/stellular/bundlrs#configuration), this will just detail Vibrant-specific configuration options.

## Containers

Vibrant works by building and serving static files from within a Docker container. To serve these assets from outside the container, we must run a specific Vibrant client within the container that sets up a socket that forwards requests from outside the container to inside. This means we do not use any ports on the host machine.
