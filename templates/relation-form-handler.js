window.addEventListener("load", (event) => {
  document.querySelectorAll(".sub-form-add-button").forEach(function (e) {
    e.addEventListener("click", function () {
      let target = e.attributes["data-target"].value;
      console.log(target);
    });
  });
  document.querySelectorAll(".sub-form-delete-button").forEach(function (e) {
    e.addEventListener("click", function () {
      let target = e.attributes["data-target"].value;
      let index = parseInt(e.attributes["data-index"].value);
      console.log(target, index);
      let w = document.getElementById(`${target}.${index}`);
      ["border", "border-2", "border-danger"].forEach(function (x) {
        w.classList.add(x);
      });
    });
  });
});
