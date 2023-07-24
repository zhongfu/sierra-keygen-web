#!/bin/bash
# lmao
echo {$(cat .dev.vars | grep -v '^#' | sed "s|=| |" | awk '{print "\"" $1 "\":\"" $2 "\","}' | tr -d "\n" | sed "s|,$||")} > secrets.json
