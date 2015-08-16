var gulp = require('gulp');
var livereload = require('gulp-livereload');

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
  console.log('notified', event);
}

gulp.task('default', function () {

  startExpress();
  livereload.listen();
  gulp.watch('html/*.html', notifyLivereload);
});
