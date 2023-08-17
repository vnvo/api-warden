#!/bin/sh
echo -n "setting up dev env ..."
echo -n "1) stop containers, if any ..."
docker compose -f ./docker-compose-devenv.yaml down
echo -n "2) start required containers ..."
docker compose -f ./docker-compose-devenv.yaml up -d
docker compose -f ./docker-compose-devenv.yaml ps
echo -n "3) create/populate required kafka topic (api-warden-aggr) ..."

echo -n "devenv setup finished."