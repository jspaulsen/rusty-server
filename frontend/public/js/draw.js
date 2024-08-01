const CANVAS = document.getElementById('canvas');
const CTX = CANVAS.getContext('2d');


const ANGLE = 2 * Math.PI / 6;
const RADIUS = 50;


class Hexagon {
  constructor(x, y) {
    this.x = x;
    this.y = y;
  }

  draw(context, color=null) {
    context.beginPath();
    
    for (var i = 0; i < 6; i++) {
      context.lineTo(
        this.x + RADIUS * Math.cos(ANGLE * i), 
        this.y + RADIUS * Math.sin(ANGLE * i),
      );
    }

    if (color) {
      context.fillStyle = color;
    }

    context.closePath();
    context.stroke();

    if (color) {
      context.fill();
    }
  }

  contains(x, y) {
    return Math.abs(this.x - x) < RADIUS && Math.abs(this.y - y) < RADIUS;
  }
}


class Grid {
  constructor(width, height) {
    this.width = width;
    this.height = height;
    this.hexagons = [];
    this.activeHexagon = null;
  }

  draw(context) {
    for (let y = RADIUS; y + RADIUS * Math.sin(ANGLE) < this.height; y += RADIUS * Math.sin(ANGLE)) {
      for (let x = RADIUS, j = 0; x + RADIUS * (1 + Math.cos(ANGLE)) < this.width; x += RADIUS * (1 + Math.cos(ANGLE)), y += (-1) ** j++ * RADIUS * Math.sin(ANGLE)) {
        const hexagon = new Hexagon(x, y);

        hexagon.draw(context);
        this.hexagons.push(hexagon);
      }
    }
  }

  findHexagon(x, y) {
    for (const hexagon of this.hexagons) {
      if (hexagon.contains(x, y)) {
        return hexagon;
      }
    }
  }

  deactivate (hexagon) {
    hexagon.draw(CTX, 'white');
    this.activeHexagon = null;
  }

  activate(hexagon) {
    this.activeHexagon = hexagon;
    hexagon.draw(CTX, 'lightblue');
  }
}


const grid = new Grid(canvas.width, canvas.height);
grid.draw(CTX);



CANVAS.addEventListener('mousemove', (event) => {
  const x = event.offsetX;
  const y = event.offsetY;

  const hexagon = grid.findHexagon(x, y);
  const activeHexagon = grid.activeHexagon;

  if (hexagon && activeHexagon === hexagon) {
    return;
  }

  if (activeHexagon && hexagon !== activeHexagon) {
    grid.deactivate(activeHexagon);
  }

  if (hexagon) {
    grid.activate(hexagon);
  }
});
