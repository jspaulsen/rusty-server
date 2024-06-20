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

ws.onmessage = onReceive;

{/* <p style="text-align: left; margin: 5px;">Jane: Hello</p>
<p style="text-align: left; margin: 5px;">Jane: Hello</p>
<p style="text-align: left; margin: 5px;">Jane: Hello</p> */}
