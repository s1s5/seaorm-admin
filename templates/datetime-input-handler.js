// Converts seconds to a string in the format "+hh:mm" or "-hh:mm"
function secondsToTimeString(seconds) {
  const sign = Math.sign(seconds) === -1 ? "-" : "+"; // Determine sign of the time string
  const absSeconds = Math.abs(seconds); // Convert to positive value for easier manipulation

  const hours = Math.floor(absSeconds / 3600); // Calculate number of hours
  const minutes = Math.floor((absSeconds % 3600) / 60); // Calculate number of minutes

  // Format the time string
  const formattedHours = String(hours).padStart(2, "0"); // Ensure two digits for hours
  const formattedMinutes = String(minutes).padStart(2, "0"); // Ensure two digits for minutes
  return sign + formattedHours + ":" + formattedMinutes;
}

window.addEventListener("load", (event) => {
  document.querySelectorAll(".datetime-input").forEach(function (e) {
    let name = e.attributes["name"].value;
    let dt = document.querySelector(`#${name}-datetime-id`);
    let se = document.querySelector(`#${name}-seconds-id`);

    function setvalue() {
      e.value = `${dt.value}:${se.value}`;
    }

    dt.addEventListener("change", setvalue);
    se.addEventListener("change", setvalue);
  });
  document
    .querySelectorAll(".datetime-input-with-timezone")
    .forEach(function (e) {
      let name = e.attributes["name"].value;
      let dt = document.querySelector(`#${name}-datetime-id`);
      let se = document.querySelector(`#${name}-seconds-id`);
      let tz = document.querySelector(`#${name}-timezone-id`);

      function setvalue() {
        let t = secondsToTimeString(Number(tz.value));
        e.value = `${dt.value}:${se.value}${t}`;
      }
      dt.addEventListener("change", setvalue);
      se.addEventListener("change", setvalue);
      tz.addEventListener("change", setvalue);
    });
});
