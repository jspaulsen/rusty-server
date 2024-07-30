// const ws = new WebSocket('wss://echo.websocket.org/');
const ws = new WebSocket('ws://localhost:3000/');

function onSend() {
    const element = document.getElementById('send');
    const message = element.value;
    
    // console.log(message);
    ws.send(message);
    element.value = '';
}

function onReceive(message) {
    const element = document.getElementById('chatbox');
    const p = document.createElement('p');

    console.log(message);

    if (message.data.includes('Request served by')) {
        return
    }

    p.style.textAlign = 'left';
    p.style.margin = '5px';
    p.innerHTML = `${message.data}`;

    element.appendChild(p);
}

// add event listener to chatbox, if enter key is pressed, send message
document.getElementById('send').addEventListener('keydown', (event) => {
    if (event.key === 'Enter') {
        onSend();
    }
});

ws.onmessage = onReceive;
