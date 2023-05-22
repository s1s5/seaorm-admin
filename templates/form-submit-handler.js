function extract_form(form_id) {
  var form = document.getElementById(form_id);

  let data = {};

  var inputs = form.getElementsByTagName("input");
  for (var i = 0; i < inputs.length; i++) {
    let input = inputs[i];
    if (input.type == "checkbox") {
      data[input.name] = input.checked;
    } else {
      data[input.name] = input.value;
    }
  }

  var textareas = form.getElementsByTagName("textarea");
  for (var i = 0; i < textareas.length; i++) {
    let textarea = textareas[i];
    data[textarea.name] = textarea.value;
  }

  var selects = form.getElementsByTagName("select");
  for (var i = 0; i < selects.length; i++) {
    let s = selects[i];
    data[s.name] = s.value;
  }

  return data;
}

function show_error(error) {
  var e = document.getElementById("form-alert");
  e.innerText = JSON.stringify(error);
  e.classList.remove("d-none");
}

function register_submit_callback(button_id, redirect_url_on_success) {
  document
    .getElementById(button_id)
    .addEventListener("click", function (event) {
      event.preventDefault();

      var submit = document.getElementById(button_id);
      submit.disabled = true;

      let data = extract_form("{{ form_id }}");
      let action = "{% if let Some(action) = action %}{{ action}}{% endif %}";
      let path = action == "" ? window.location.pathname : action;
      fetch(path, {
        method: "{{ method }}",
        body: JSON.stringify(data),
        headers: {
          "Content-Type": "application/json",
        },
      })
        .then(async function (response) {
          let data = await response.json();
          if (response.ok) {
            return data;
          } else {
            throw data;
          }
        })
        .then(function (data) {
          if (typeof redirect_url_on_success === "function") {
            window.location.href = redirect_url_on_success(data);
          } else {
            window.location.href = redirect_url_on_success;
          }
        })
        .catch(function (error) {
          var myDiv = document.getElementById("form-alert");
          // myDiv.innerText = JSON.stringify(error.error);
          myDiv.innerText = error.error;

          myDiv.classList.remove("d-none");
          console.error(error); // handle error
        })
        .finally(function () {
          submit.disabled = false;
        });
    });
}
