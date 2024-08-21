#!/bin/bash
--set -x
# Example file on how to build different configurations of the rusty-bridge

# Choose which configuration logic to build (this can be multiple)
# Options: 
# * dev
# * special
MINI_CONFIG="dev"

# Choose which message busses to build (this can be multiple)
# ! Multiple busses are not supported yet !
# Options: 
# * dev
# * special
# * http-rest
# * bacnet -- experimental
DATA_SOURCES="dev"

# Choose which message transform to use. The message transform transforms the message received from the bus, to something compatible for the cloud
# Options:
# * dev
# * special
MSG_TRANSFORM="dev"

# Choose which cloud adapters to build and have available (this can be multiple)
# ! Multiple cloud adapters are not supported yet !
# Options:
# * dev
# * special-hivemq
# * iothub
CLOUD_ADAPTERS="special-hivemq"

# Choose which persistence method to use
# Options:
# * dev
# * sled
PERSISTENCE="dev"

#---

##### Optional #####
# Enable compiling in the data server. This is necessary to enable serving up rusty-bridge metrics/data to consuming clients/viewers
# Comment it out if you do not want to compile the data-server in (300-400 KB)
# Options:
# * data-server
# * none
DATA_SERVER="data-server"




# Pleasantries in echo colors
RED='\033[0;31m'
GREEN='\033[0;32m'
LIGHT_GREEN='\033[1;32m'
NC='\033[0m' # No Color


echo "Building with the following feature flags..."
MINI_CONFIG="mini-config/$MINI_CONFIG"
DATA_SOURCES="data-source/$DATA_SOURCES"
MSG_TRANSFORM="msg-transforms/$MSG_TRANSFORM"
CLOUD_ADAPTERS="cloud-adapter/$CLOUD_ADAPTERS"
PERSISTENCE="msg-persistence/$PERSISTENCE"
#---
if [ -z ${DATA_SERVER+x}]; 
  then echo "DATA_SERVER not set, it wont be compiled in";
  else DATA_SERVER="rusty-bridge/$DATA_SERVER";
fi

FEATURES="$DATA_SOURCES,$MSG_TRANSFORM,$CLOUD_ADAPTERS,$PERSISTENCE,$DATA_SERVER,$MINI_CONFIG"
RUSTFLAGS="-Awarnings" cargo build --no-default-features --features="$FEATURES" --release --bin rusty-bridge
echo "Flags: $FEATURES"
echo -e "${GREEN}Build complete${NC}"
SIZE=$(du -h ../target/release/rusty-bridge)
echo -e "Build size: ${LIGHT_GREEN}$SIZE${NC}"
#./target/
