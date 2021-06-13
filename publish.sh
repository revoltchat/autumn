#!/bin/bash
source set_version.sh

docker build -t revoltchat/autumn:${version} . &&
    docker push revoltchat/autumn:${version}
