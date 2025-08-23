let input = document.getElementById("chat-input");
let chatBox = document.getElementById("chat");
let socket = new WebSocket("wss://chat.def1de.com/ws");
let username_field = document.getElementById("username");
let user_count_field = document.getElementById("user-count");

socket.onopen = function () {
    updateUserCount();
};

socket.onclose = function () {
    socket = new WebSocket("wss://chat.def1de.com/ws");
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

function updateUserCount() {
    fetch("https://chat.def1de.com/status")
        .then((response) => response.json())
        .then((data) => {
            user_count_field.innerText = "Current users: " + data.connected_clients;
        })
        .catch((error) => {
            console.error("Error fetching status:", error);
            user_count_field.innerText = "Current users: 0";
        });
}

setInterval(updateUserCount, 10000);
