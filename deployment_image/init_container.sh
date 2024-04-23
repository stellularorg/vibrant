#!/usr/bin/env bash

# pull container id
# short will do
export CID=$HOSTNAME

# pull build script
# the container should be run with "--net=host" and with the "VIBRANT_URL" var pointing to the http url of the vibrant server
# the environment variable for "PROJECT" should also be set to the project id
curl $VIBRANT_URL/api/v1/$PROJECT/script -o build.sh

# move to ./app
mkdir app
cd app

# run build script
chmod +x build.sh
./build.sh

# remove build script
rm build.sh
