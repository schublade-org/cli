(function () {
  var prefix = "";
  var scripts = document.getElementsByTagName("script");
  for (var i = 0; i < scripts.length; i++) {
    var src = scripts[i].getAttribute("src") || "";
    var index = src.lastIndexOf("workshop-boot.js");
    if (index !== -1) {
      prefix = src.slice(0, index);
      break;
    }
  }
  Promise.all(
    ["workshop.0.js", "workshop.1.js", "workshop.2.js"].map(function (name) {
      return fetch(prefix + name).then(function (response) {
        if (!response.ok) throw new Error(name);
        return response.text();
      });
    })
  )
    .then(function (parts) {
      var script = document.createElement("script");
      script.text = parts.join("");
      document.body.appendChild(script);
    })
    .catch(function (error) {
      console.error("Could not load the workshop chrome.", error);
    });
})();
