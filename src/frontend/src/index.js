import { backend } from "../../declarations/backend";

document.getElementById("classify").onclick = classify;
document.getElementById("file").onchange = onImageChange;

// Calls the backend to perform image classification.
async function classify(event) {
  event.preventDefault();

  const button = event.target;
  const message = document.getElementById("message");
  const loader = document.getElementById("loader");
  const img = document.getElementById("image");
  const repl_option = document.getElementById("replicated_option");

  button.disabled = true;
  button.className = "clean-button invisible";
  repl_option.className = "option invisible";
  message.innerText = "Computing...";
  loader.className = "loader";

  try {
    const blob = await resize(img);
    let result;
    if (document.getElementById("replicated").checked) {
      result = await backend.classify(new Uint8Array(blob));
    } else {
      result = await backend.classify_query(new Uint8Array(blob));
    }
    if (result.Ok) {
        try {
            result = await backend.llm(result.Ok[0].label)
            result = JSON.parse(result.slice(0, -90))
            render(message, result.choices[0].message.content);
        } catch (err) {
            message.innerText = "Failed to call openAI: " + JSON.stringify(err);
        }
    } else {
      throw result.Err;
    }
  } catch (err) {
    message.innerText = "Failed to classify image: " + JSON.stringify(err);
  }
  loader.className = "loader invisible";

  return false;
}

// Resizes the given image to 224x224px and returns the resulting PNG blob.
async function resize(img) {
  const canvas = document.createElement("canvas");
  canvas.width = 224;
  canvas.height = 224;
  let scale = Math.max(canvas.width / img.naturalWidth, canvas.height / img.naturalHeight);
  let width = img.naturalWidth * scale;
  let height = img.naturalHeight * scale;
  let x = canvas.width / 2 - width / 2;
  let y = canvas.height / 2 - height / 2;
  const ctx = canvas.getContext("2d");
  if (ctx) {
    ctx.drawImage(img, x, y, width, height);
  }
  let bytes = await serialize(canvas);
  return bytes;
}

// Serializes the given canvas into PNG image bytes.
function serialize(canvas) {
  return new Promise((resolve) => canvas.toBlob((blob) => blob.arrayBuffer().then(resolve), "image/png", 0.9));
}

// Adds the classification results as a list to the given DOM element.
function render(element, result) {
  element.innerText = "Results:";
  let p = document.createElement("p");
  p.textContent = result;
  p.setAttribute('class', 'result');
  element.appendChild(p);
}

// This function is called when the user selects a new image file.
async function onImageChange(event) {
  const button = document.getElementById("classify");
  const message = document.getElementById("message");
  const img = document.getElementById("image");
  const repl_option = document.getElementById("replicated_option");
  try {
    const file = event.target.files[0];
    const url = await toDataURL(file);
    img.src = url;
    img.width = 600;
    img.className = "image";
  } catch (err) {
    message.innerText = "Failed to select image: " + err.toString();
  }
  button.disabled = false;
  button.className = "clean-button";
  message.innerText = "";
  repl_option.className = "option"
  return false;
}

// Converts the given blob into a data url such that it can be assigned as a
// target of a link of as an image source.
function toDataURL(blob) {
  return new Promise((resolve, _) => {
    const fileReader = new FileReader();
    fileReader.readAsDataURL(blob);
    fileReader.onloadend = function () {
      resolve(fileReader.result);
    }
  });
}
