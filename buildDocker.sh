#!/bin/sh

export DOCKER_BUILDKIT=1

docker build -t huydinheeit/gotify_sample . && docker push huydinheeit/gotify_sample