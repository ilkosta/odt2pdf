var gulp = require('gulp');
var livereload = require('gulp-livereload');
var jshint = require('gulp-jshint');

var EXPRESS_PORT = 4000;
var EXPRESS_ROOT = __dirname;
var LIVERELOAD_PORT = 35729;

// Let's make things more readable by
// encapsulating each part's setup
// in its own method
function startExpress() {

  var express = require('express');
  var app = express();
  app.use(require('connect-livereload')());
  app.use(express.static(EXPRESS_ROOT + '/html/'));
  app.listen(EXPRESS_PORT);

  console.log('server started at port ' + EXPRESS_PORT);
}


// Notifies livereload of changes detected
// by `gulp.watch()`
function notifyLivereload(event) {

  gulp.src(event.path, {read: false})
      .pipe(livereload({ start: true }));
}

function lint(event) {
  return gulp.src('./html/js/*.js')
    .pipe(jshint())
    .pipe(jshint.reporter('jshint-stylish'));
}


gulp.task('default', function () {

  startExpress();
  livereload.listen();
  gulp.watch('html/*.html', notifyLivereload);
  gulp.watch('html/js/*.js', lint);
});
