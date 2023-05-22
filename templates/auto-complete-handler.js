function debounce(func, timeout) {
  let timer;
  return function (...args) {
    clearTimeout(timer);
    timer = setTimeout(() => {
      func.apply(this, args);
    }, timeout);
  };
}

async function get_choices(to_table, query, nullable) {
  let res = await fetch(
    query == null
      ? `{{ site.sub_path }}/${to_table}/`
      : `{{ site.sub_path }}/${to_table}/?_q=${query}`,
    {
      headers: {
        accept: "application/json",
      },
    }
  );
  let object_list = await res.json();
  return (
    nullable
      ? [
          {
            label: "&lt; clear value &gt;",
            value: "",
          },
        ]
      : []
  ).concat(
    object_list.data.map((e) => ({
      label: e.label,
      value: e.key,
      customProperties: {
        data: e.data,
      },
    }))
  );
}

window.addEventListener("load", (event) => {
  document.querySelectorAll(".auto-complete").forEach(function (e) {
    let to_table = e.attributes["data-to_table"].value;
    let nullable =
      e.attributes["data-nullable"] == null
        ? false
        : e.attributes["data-nullable"].value;
    let relations = [0, 1, 2]
      .map((i) => [
        e.attributes[`data-${i}-to`],
        e.attributes[`data-${i}-from`],
      ])
      .filter((e) => !!e[0])
      .map((e) => [e[0].value, e[1].value]);

    let choices = new Choices(e, {
      choices: [],
      shouldSort: false,
      searchChoices: false,
    });
    (async () => {
      choices.setChoices(
        await get_choices(to_table, null, nullable),
        "value",
        "label",
        true
      );
    })();

    let on_search = debounce(async function (event) {
      choices.setChoices(
        await get_choices(to_table, event.detail.value, nullable),
        "value",
        "label",
        true
      );
    }, 100);

    e.addEventListener("search", on_search, false);

    e.addEventListener(
      "addItem",
      function (event) {
        if (event.detail.customProperties == null) {
          relations.forEach((rel) => {
            let target_el = document.querySelector(`#${rel[1]}-id`);
            target_el.value = "";
          });
        } else {
          let data = event.detail.customProperties.data;
          relations.forEach((rel) => {
            let target_el = document.querySelector(`#${rel[1]}-id`);
            target_el.value = data[rel[0]];
          });
        }
      },
      false
    );
  });
});
