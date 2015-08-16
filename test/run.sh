#!/bin/sh


#./make_curl.sh   "/home/costa/Modulo Cohesion - Enti-1.odt"


#locate -r '\.odt$' > ../odt.list

cat ../odt.list | xargs -I {} ./make_curl.sh {}
cat ../odt.list | xargs -I {} ./make_curl.sh {}

while [ $? -eq 0 ]
do
  sleep 2
  pgrep curl > /dev/null
done
