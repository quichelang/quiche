type Position:
    x: i64
    y: i64


type BumbleBee:
    position: Position
    previous: Position

    def move(self, new_position: Position):
        self.previous = self.position
        self.position = new_position

        # Determine the direction of movement
        if self.position.x > self.previous.x:
            print("Moving right")
        elif self.position.x < self.previous.x:
            print("Moving left")
        
        if self.position.y > self.previous.y:
            print("Moving up")
        elif self.position.y < self.previous.y:
            print("Moving down")

def main():
    v = [1, 2, 3, 4, 5]
    q = [5, 6, 7, 7, 9]

    for x in range(10):
        for y in range(5):
            print(x, y)
    