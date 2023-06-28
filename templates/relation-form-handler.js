window.addEventListener("load", (event) => {
  function create_row(target) {
    let state_el = document.getElementById(`${target}.state-id`);
    let l = state_el.value.split(",");
    let index = l.length;
    l.push("C");
    state_el.value = l.join(",");
    return index;
  }

  function handle_delete(e) {
    let target = e.attributes["data-target"].value;
    let index = parseInt(e.attributes["data-index"].value);
    let state_el = document.getElementById(`${target}.state-id`);
    let l = state_el.value.split(",");
    let w = document.getElementById(`${target}.${index}`);
    if (l[index] == "U") {
      l[index] = "D";
      ["border", "border-2", "border-danger"].forEach(function (x) {
        w.classList.add(x);
      });
      e.textContent = "restore";
    } else if (l[index] == "D") {
      l[index] = "U";
      ["border", "border-2", "border-danger"].forEach(function (x) {
        w.classList.remove(x);
      });
      e.textContent = "del";
    } else {
      l[index] = "I";
      w.classList.add("d-none");
    }
    state_el.value = l.join(",");
  }

  function walk_recursive(e, f) {
    f(e);
    e.childNodes.forEach(function (c) {
      walk_recursive(c, f);
    });
  }

  function update_input_name(index, e) {
    if (!e.attributes) {
      return;
    }

    for (let i = 0; i < e.attributes.length; i++) {
      let a = e.attributes[i];
      e.setAttribute(a.name, a.value.replace("${index}", index));
    }
  }

  document.querySelectorAll(".sub-form-add-button").forEach(function (e) {
    e.addEventListener("click", function () {
      let target = e.attributes["data-target"].value;
      let index = create_row(target);

      let cloned = document
        .getElementById(`${target}-template`)
        .cloneNode(true);
      cloned.getRootNode().classList.remove("d-none");
      let root = cloned.getRootNode();
      root.setAttribute("id", `${target}.${index}`);
      let b = root.querySelector(".sub-form-delete-button");
      b.setAttribute("data-index", index);
      b.addEventListener("click", function () {
        handle_delete(b);
      });
      walk_recursive(root, function (e) {
        update_input_name(index, e);
      });

      document.getElementById(`${target}-container`).append(cloned);
    });
  });
  document.querySelectorAll(".sub-form-delete-button").forEach(function (e) {
    e.addEventListener("click", function () {
      handle_delete(e);
    });
  });
});
