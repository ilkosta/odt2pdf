function handleFile(selecteFile) {

  if(selecteFile) {
    document.getElementById('size').innerHTML = "dimensione: " + filesize(selecteFile.size)

    var reader = new FileReader();
    var sha1 = CryptoJS.algo.SHA1.create();
    var read = 0;
    var unit = 1024 * 1024;
    var blob;
    var reader = new FileReader();
    reader.readAsArrayBuffer(selecteFile.slice(read, read + unit));
    reader.onload = function(e) {
        var bytes = CryptoJS.lib.WordArray.create(e.target.result);
        sha1.update(bytes);
        read += unit;
        if (read < selecteFile.size) {
            blob = selecteFile.slice(read, read + unit);
            reader.readAsArrayBuffer(blob);
        } else {
            var hash = sha1.finalize();
            console.log('secondo metodo: ', hash.toString(CryptoJS.enc.Hex)); // print the result
            document.getElementById('sha1sum').value = hash.toString(CryptoJS.enc.Hex);
        }
    }        
    
  } else {
    document.getElementById('size').innerHTML = "";
    document.getElementById('sha1sum').value = "";
  }
}