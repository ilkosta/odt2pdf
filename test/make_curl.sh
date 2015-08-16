#!/bin/sh

fname="$@"
#echo "fname = $fname"

#echo md5sum = $(md5sum "$fname")
md5sum=$(md5sum "$fname" | sed -E s/'[[:space:]]+.+'//)
#echo "md5sum = $md5sum"

curl -s -o docs/"$md5sum".pdf -i "http://localhost:3000/openact"  -F "filename=@$fname" -F "md5sum=$md5sum" &
echo eseguito: curl -i "http://localhost:3000/openact"  -F "\"filename=@$fname\"" -F "\"md5sum=$md5sum\""
