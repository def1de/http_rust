let input = document.getElementById("chat-input");
let chatBox = document.getElementById("chat");
let socket = new WebSocket("wss://chat.def1de.com/ws");

socket.onopen = function () {
    let name = prompt("Enter your name:");
    if (!name || name.trim() === "") {
        name = "Anonymous";
    }

    socket.send(name);
};

socket.onclose = function () {
    alert("Connection is closed...");
};

input.addEventListener("keydown", function (event) {
    if (event.key === "Enter") {
        event.preventDefault();
        const message = input.value;
        if (message.trim() !== "") {
            socket.send(message);

            input.value = "";
        }

        chatBox.innerHTML +=
            `<div class="message right">
                    <p class="username">You</p>
                    <p class="message_content">` +
            message +
            `</p>
                </div>`;
        scrollToBottom();
    }
});

socket.onmessage = (event) => {
    const parts = event.data.split(": ", 2);
    chatBox.innerHTML +=
        `<div class="message left">
                    <p class="username">` +
        parts[0] +
        `</p>
                    <p class="message_content">` +
        parts[1] +
        `</p>
                </div>`;
    scrollToBottom();
};

function scrollToBottom() {
    chatBox.scrollTo({
        top: chatBox.scrollHeight,
        behavior: "smooth",
    });
}
