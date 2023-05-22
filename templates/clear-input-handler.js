window.addEventListener("load", (event) => {
  document.querySelectorAll(".clear-input-button").forEach(function (e) {
    e.addEventListener("click", function () {
      let ids = e.attributes["data-target"].value.split(",").filter((e) => !!e);
      ids.forEach(function (id) {
        document.querySelectorAll("#" + id).forEach(function (e) {
          e.value = "";
        });
      });
    });
  });
});
