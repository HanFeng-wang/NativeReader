const button = document.querySelector("#demo");
const result = document.querySelector("#result");

button.addEventListener("click", () => {
  result.textContent = `JavaScript 已运行：${new Date().toLocaleTimeString()}`;
});
