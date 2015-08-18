# odt2pdf
webservice REST to convert odt documents to pdf

# don't use it!

The software is only a learning test for my knowledge of rust and it's ecosystem.

It's the first rust project that I've created to test my learning.

Consider it **pre-alpha quality**.

## dependencies

* libreoffice
* coreutils (for the use of `timeout` changable by the configuration file when it will work)
* time (changable by the configuration file when it will work)
* [iron](https://github.com/iron/iron) / [params](https://github.com/iron/params) / [router](https://github.com/iron/router) / [staticfile](https://github.com/iron/static) / [mount](https://github.com/iron/mount)
* [fern](https://github.com/daboross/fern-rs) / log
* config

To generate the sample html form:

* node/iojs
* bower


## build the static form

From the project directory 

``` shell
cd src/asset
nvm use # if you use nvm to switch between different node/iojs versions
npm install
bower install
npm run jade
```

## run the service

```shell
cargo run
```

`POST` the document to http://localhost:3000/odt2pdf by using a multipart/formdata request.

For an sample form `GET` http://localhost:3000/openact
