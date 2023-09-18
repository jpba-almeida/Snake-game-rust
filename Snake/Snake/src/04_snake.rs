//! Este código foi baseado no tutorial "Making a Snake Game in Rust":
//! <https://www.youtube.com/watch?v=HCwMb0KslX8>
//! para mostrar o uso do Piston e dar um exemplo de gráficos 2D em Rust.
//!
//! O código que serviu de inspiração para esta versão foi criado usando GGEZ e adaptado da implementação do jogo criado por @termhn.
//! Original repo: <https://github.com/termhn/ggez_snake>
//!
//! O jogo foi feito seguindo o tutorial do vídeo "Making a Snake Game in Rust" e adaptado por mim, João Paulo Brito de Almeida, com inspiração no jogo criado por @termhn.
//!
//! Autor: João Paulo Brito de Almeida
//!
//! ---

// Importações de bibliotecas e módulos necessários
use oorandom::Rand32;
use ggez::{
    event, graphics,
    input::keyboard::{KeyCode, KeyInput},
    Context, GameResult,
};
use std::collections::VecDeque;

// Constantes para o tamanho da grade, tamanho da célula da grade, tamanho da tela e taxa de quadros por segundo desejada.
const GRID_SIZE: (i16, i16) = (30, 20);
const GRID_CELL_SIZE: (i16, i16) = (32, 32);
const SCREEN_SIZE: (f32, f32) = (
    GRID_SIZE.0 as f32 * GRID_CELL_SIZE.0 as f32,
    GRID_SIZE.1 as f32 * GRID_CELL_SIZE.1 as f32,
);
const DESIRED_FPS: u32 = 8;

// Estrutura para representar uma posição na grade.
struct GridPosition {
    x: i16,
    y: i16,
}

// Implementações para a estrutura GridPosition.
impl GridPosition {
    // Cria uma nova posição na grade com coordenadas x e y.
    pub fn new(x: i16, y: i16) -> Self {
        GridPosition { x, y }
    }

    // Cria uma posição na grade aleatória dentro dos limites especificados.
    pub fn random(rng: &mut Rand32, max_x: i16, max_y: i16) -> Self {
        (
            rng.rand_range(0..(max_x as u32)) as i16,
            rng.rand_range(0..(max_y as u32)) as i16,
        )
            .into()
    }

    // Cria uma nova posição na grade com base em uma posição atual e uma direção de movimento.
    pub fn new_from_move(pos: GridPosition, dir: Direction) -> Self {
        match dir {
            Direction::Up => GridPosition::new(pos.x, (pos.y - 1).rem_euclid(GRID_SIZE.1)),
            Direction::Down => GridPosition::new(pos.x, (pos.y + 1).rem_euclid(GRID_SIZE.1)),
            Direction::Left => GridPosition::new((pos.x - 1).rem_euclid(GRID_SIZE.0), pos.y),
            Direction::Right => GridPosition::new((pos.x + 1).rem_euclid(GRID_SIZE.0), pos.y),
        }
    }
}

// Implementações para conversão de GridPosition em um retângulo de gráficos.
impl From<GridPosition> for graphics::Rect {
    fn from(pos: GridPosition) -> Self {
        graphics::Rect::new_i32(
            pos.x as i32 * GRID_CELL_SIZE.0 as i32,
            pos.y as i32 * GRID_CELL_SIZE.1 as i32,
            GRID_CELL_SIZE.0 as i32,
            GRID_CELL_SIZE.1 as i32,
        )
    }
}

// Implementação de conversão de tupla (i16, i16) para GridPosition.
impl From<(i16, i16)> for GridPosition {
    fn from(pos: (i16, i16)) -> Self {
        GridPosition { x: pos.0, y: pos.1 }
    }
}

// Enumeração para representar direções possíveis.
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

// Implementações para a enumeração Direction.
impl Direction {
    // Retorna a direção oposta.
    pub fn inverse(self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    // Converte um KeyCode em uma direção, se possível.
    pub fn from_keycode(key: KeyCode) -> Option<Direction> {
        match key {
            KeyCode::Up => Some(Direction::Up),
            KeyCode::Down => Some(Direction::Down),
            KeyCode::Left => Some(Direction::Left),
            KeyCode::Right => Some(Direction::Right),
            _ => None,
        }
    }
}

// Estrutura para representar um segmento da Snake.
struct Segment {
    pos: GridPosition,
}

// Implementações para a estrutura Segment.
impl Segment {
    // Cria um novo segmento com uma posição na grade.
    pub fn new(pos: GridPosition) -> Self {
        Segment { pos }
    }
}

// Estrutura para representar a comida.
struct Food {
    pos: GridPosition,
}

// Implementações para a estrutura Food.
impl Food {
    // Cria um novo objeto de comida com uma posição na grade.
    pub fn new(pos: GridPosition) -> Self {
        Food { pos }
    }

    // Desenha a comida no canvas.
    fn draw(&self, canvas: &mut graphics::Canvas) {
        let color = [0.0, 0.0, 1.0, 1.0];
        canvas.draw(
            &graphics::Quad,
            graphics::DrawParam::new()
                .dest_rect(self.pos.into())
                .color(color),
        );
    }
}

// Enumeração para representar o que a Snake comeu.
enum Ate {
    Itself,
    Food,
}

// Estrutura para representar a Snake.
struct Snake {
    head: Segment,
    dir: Direction,
    body: VecDeque<Segment>,
    ate: Option<Ate>,
    last_update_dir: Direction,
    next_dir: Option<Direction>,
}

// Implementações para a estrutura Snake.
impl Snake {
    // Cria uma nova Snake com uma posição inicial na grade.
    pub fn new(pos: GridPosition) -> Self {
        let mut body = VecDeque::new();
        body.push_back(Segment::new((pos.x - 1, pos.y).into()));
        Snake {
            head: Segment::new(pos),
            dir: Direction::Right,
            last_update_dir: Direction::Right,
            body,
            ate: None,
            next_dir: None,
        }
    }

    // Verifica se a Snake comeu a comida.
    fn eats(&self, food: &Food) -> bool {
        self.head.pos == food.pos
    }

    // Verifica se a Snake comeu a si mesma.
    fn eats_self(&self) -> bool {
        for seg in &self.body {
            if self.head.pos == seg.pos {
                return true;
            }
        }
        false
    }

    // Atualiza o estado da Snake.
    fn update(&mut self, food: &Food) {
        if self.last_update_dir == self.dir && self.next_dir.is_some() {
            self.dir = self.next_dir.unwrap();
            self.next_dir = None;
        }
        let new_head_pos = GridPosition::new_from_move(self.head.pos, self.dir);
        let new_head = Segment::new(new_head_pos);
        self.body.push_front(self.head);
        self.head = new_head;
        if self.eats_self() {
            self.ate = Some(Ate::Itself);
        } else if self.eats(food) {
            self.ate = Some(Ate::Food);
        } else {
            self.ate = None;
        }
        if self.ate.is_none() {
            self.body.pop_back();
        }
        self.last_update_dir = self.dir;
    }

    // Desenha a Snake no canvas.
    fn draw(&self, canvas: &mut graphics::Canvas) {
        for seg in &self.body {
            canvas.draw(
                &graphics::Quad,
                graphics::DrawParam::new()
                    .dest_rect(seg.pos.into())
                    .color([0.3, 0.3, 0.0, 1.0]),
            );
        }
        canvas.draw(
            &graphics::Quad,
            graphics::DrawParam::new()
                .dest_rect(self.head.pos.into())
                .color([1.0, 0.5, 0.0, 1.0]),
        );
    }
}

// Estrutura para representar o estado do jogo.
struct GameState {
    snake: Snake,
    food: Food,
    gameover: bool,
    rng: Rand32,
}

// Implementações para a estrutura GameState.
impl GameState {
    // Cria um novo estado de jogo.
    pub fn new() -> Self {
        let snake_pos = (GRID_SIZE.0 / 4, GRID_SIZE.1 / 2).into();
        let mut seed: [u8; 8] = [0; 8];
        getrandom::getrandom(&mut seed[..]).expect("Could not create RNG seed");
        let mut rng = Rand32::new(u64::from_ne_bytes(seed));
        let food_pos = GridPosition::random(&mut rng, GRID_SIZE.0, GRID_SIZE.1);

        GameState {
            snake: Snake::new(snake_pos),
            food: Food::new(food_pos),
            gameover: false,
            rng,
        }
    }
}

// Implementações do EventHandler para GameState.
impl event::EventHandler<ggez::GameError> for GameState {
    // Função de atualização do jogo.
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while ctx.time.check_update_time(DESIRED_FPS) {
            if !self.gameover {
                self.snake.update(&self.food);
                if let Some(ate) = self.snake.ate {
                    match ate {
                        Ate::Food => {
                            let new_food_pos =
                                GridPosition::random(&mut self.rng, GRID_SIZE.0, GRID_SIZE.1);
                            self.food.pos = new_food_pos;
                        }
                        Ate::Itself => {
                            self.gameover = true;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    // Função para desenhar o estado atual do jogo.
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from([0.0, 1.0, 0.0, 1.0]));
        self.snake.draw(&mut canvas);
        self.food.draw(&mut canvas);
        canvas.finish(ctx)?;
        ggez::timer::yield_now();
        Ok(())
    }

    // Função para lidar com eventos de teclado.
    fn key_down_event(&mut self, _ctx: &mut Context, input: KeyInput, _repeat: bool) -> GameResult {
        if let Some(dir) = input.keycode.and_then(Direction::from_keycode) {
            if self.snake.dir != self.snake.last_update_dir && dir.inverse() != self.snake.dir {
                self.snake.next_dir = Some(dir);
            } else if dir.inverse() != self.snake.last_update_dir {
                self.snake.dir = dir;
            }
        }
        Ok(())
    }
}

// Função principal do programa.
fn main() -> GameResult {
    let (ctx, events_loop) = ggez::ContextBuilder::new("snake", "Gray Olson")
        .window_setup(ggez::conf::WindowSetup::default().title("Snake!"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
        .build()?;
    let state = GameState::new();
    event::run(ctx, events_loop, state)
}
