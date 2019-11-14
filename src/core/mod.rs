use ggez::*;
use torifune::core::Clock;

pub struct State {
    clock: Clock,
    fps: f64,
}

impl State {
    pub fn new(ctx: &mut Context) -> GameResult<State> {
        let s = State {
            clock: 0,
            fps: 0.0,
        };

        Ok(s)
    }
}

impl ggez::event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.clock += 1;
        if (self.clock % 100) == 0 {
            self.fps = timer::fps(ctx);
        }

        Ok(())
    }
    
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, [0.0, 0.0, 0.0, 0.0].into());

        graphics::present(ctx)?;

        Ok(())
    }
}
