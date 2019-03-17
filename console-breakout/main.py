import math
import sys
from collections import namedtuple
from curses import wrapper
import time

HEIGHT = 30
WIDTH = 120

WIDTH_PLAYER = 5
WIDTH_BRICK = 5

NUM_BRICK_LINES = 6

V_MAX_PLAYER = 9
V_MAX_BALL = 10

A_MAX_PLAYER = 40
A_GAIN_PLAYER = 30


GameState = namedtuple('GameState', ['player', 'ball', 'bricks'])


class GameObj(object):
    WIDTH = 1
    HEIGHT = 1

    def __init__(self, x, y):
        self.x = x
        self.y = y

    def place_on(self, screen):
        x_i = int(self.x)
        y_i = int(self.y)
        for y in range(y_i, min(y_i+self.HEIGHT, HEIGHT)):
            for x in range(x_i, min(x_i+self.WIDTH, WIDTH)):
                screen[y][x] = self

    def pos(self):
        return self.x, self.y


class Player(GameObj):
    WIDTH = WIDTH_PLAYER

    V_MAX = V_MAX_PLAYER
    A_MAX = A_MAX_PLAYER

    def __init__(self, x, y=HEIGHT-3):
        GameObj.__init__(self, x, y)
        self.v = 0
        self.a = 0

    def update(self, dt):
        self.v += self.a * dt
        if abs(self.v) > self.V_MAX:
            self.v = math.copysign(self.V_MAX, self.v)
            self.a = 0

        self.x += self.v * dt
        if self.x <= 0:
            self.x = 0
            self.v = 0
            self.a = 0
        elif self.x >= WIDTH - self.WIDTH:
            self.x = WIDTH - self.WIDTH
            self.v = 0
            self.a = 0

    def move_left(self):
        if self.a > 0:  # let's cheat a bit
            self.a = 0
        self.a = -min(self.A_MAX, abs(self.a) + A_GAIN_PLAYER)

    def move_right(self):
        if self.a < 0:
            self.a = 0
        self.a = min(self.A_MAX, abs(self.a) + A_GAIN_PLAYER)

    def __str__(self):
        return 'x'


class Ball(GameObj):
    WIDTH = 1

    V_MAX = V_MAX_BALL

    def __init__(self, x, y):
        GameObj.__init__(self, x, y)
        self.v = (0, -self.V_MAX)

    def update(self, dt):
        self.x += self.v[0] * dt
        self.y += self.v[1] * dt

        if self.x <= 0:
            self.x = 0
            self.set_v(-self.v[0], self.v[1])
        elif self.x >= WIDTH - 1:
            self.x = WIDTH - 1
            self.set_v(-self.v[0], self.v[1])

        if self.y <= 0:
            self.y = 0
            self.set_v(-self.v[0], -self.v[1])
        elif self.y >= HEIGHT - 1:
            sys.exit(1)

    def set_v(self, vx, vy):
        v = math.sqrt(vx*vx + vy*vy)
        if v > self.V_MAX:
            vx = vx / v * self.V_MAX
            vy = vy / v * self.V_MAX

        self.v = (vx, vy)

    def __str__(self):
        return 'o'


class Brick(GameObj):
    WIDTH = WIDTH_BRICK

    def __init__(self, x, y):
        GameObj.__init__(self, x, y)

    def __str__(self):
        return '='


def reset():
    y = HEIGHT - 3
    x = WIDTH // 2 - Player.WIDTH
    player = Player(x, y)

    y -= 1
    x += player.WIDTH // 2
    ball = Ball(x, y)

    bricks = []
    for y in range(NUM_BRICK_LINES):
        for x in range(0, WIDTH-WIDTH_BRICK, WIDTH_BRICK+1):
            bricks.append(Brick(x, y))

    return GameState(player, ball, bricks)


def update_screen(state):
    screen = [ [ ' ' for _ in range(WIDTH) ] for _ in range(HEIGHT) ]

    state.player.place_on(screen)
    state.ball.place_on(screen)
    for brick in state.bricks:
        brick.place_on(screen)

    return screen


def draw(scr, screen):
    scr.clear()
    for line in screen:
        scr.addstr(''.join(map(str, line)) + '\n')
    scr.refresh()


def update(state, screen, dt):

    old_ball_pos = state.ball.pos()

    state.player.update(dt)
    state.ball.update(dt)

    x, y = map(int, state.ball.pos())
    neighbors = []
    for y_ in range(max(0, y-1), min(HEIGHT, y+1)):
        for x_ in range(max(0, x-1), min(WIDTH, x+1)):
            neighbors.append(screen[y_][x_])
    neighbors = set(neighbors)
    neighbors = list(filter(lambda el: type(el) != str and type(el) != Ball, neighbors))

    while neighbors:
        tmp = neighbors.pop()

        if type(tmp) == Brick:
            state.bricks.remove(tmp)

        vx, vy = state.ball.v

        # introduce some shift
        cx, _ = tmp.pos()
        cx += tmp.WIDTH // 2
        dx = x - cx
        vx -= dx

        state.ball.set_v(-vx, -vy)

        state.ball.x = old_ball_pos[0]
        state.ball.y = old_ball_pos[1]

    return update_screen(state)


def game_loop(scr):
    dt = 0.05

    scr.nodelay(True)

    state = reset()
    screen = update_screen(state)

    while True:
        screen = update(state, screen, dt)
        draw(scr, screen)

        inp = scr.getch()
        if inp == ord('h'):
            state.player.move_left()
        elif inp == ord('l'):
            state.player.move_right()

        if len(state.bricks) == 0:
            scr.nodelay(False)
            scr.clear()
            scr.addstr('YOU WIN!')
            scr.refresh()
            state.getch()
            break

        time.sleep(dt)


if __name__ == "__main__":
    wrapper(game_loop)
