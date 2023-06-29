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
  return object_list.data.map((e) => ({
    label: e.label,
    value: e.key,
    customProperties: {
      data: e.data,
    },
  }));
}

window.addEventListener("load", (event) => {
  document.querySelectorAll(".auto-complete").forEach(function (e) {
    // let memory = {};
    console.log(e.attributes["multiple"]);
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

      remoteItems: true,
      removeItemButton: true,
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
    e.addEventListener("removeItem", function (event) {
      // console.log("remove:", event);
      let data = event.detail.customProperties.data;
      relations.forEach((rel) => {
        let target_el = document.querySelector(`#${rel[1]}-id`);
        target_el.value = target_el.value
          .split(",")
          .filter((i) => !!i)
          .filter((i) => i != [data[rel[0]]])
          .join(",");
      });
    });

    e.addEventListener(
      "addItem",
      function (event) {
        let data = event.detail.customProperties.data;

        relations.forEach((rel) => {
          let target_el = document.querySelector(`#${rel[1]}-id`);
          target_el.value = target_el.value
            .split(",")
            .filter((i) => !!i)
            .concat([data[rel[0]]])
            .join(",");
        });
      },
      false
    );
    e.addEventListener("hideDropdown", function (event) {
      // console.log("hideDropdown", event);
      (async () => {
        choices.setChoices(
          await get_choices(to_table, null, nullable),
          "value",
          "label",
          true
        );
      })();
    });
  });
});
