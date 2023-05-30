use super::ConnectOption;
use actix::prelude::*;

pub struct ActixWsClient {
    option: ConnectOption,
}

impl Actor for ActixWsClient {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        self.option.clone()
            .connect_tokio()
            .into_actor(self)
            .map(|conn, cli, ctx| {
                match conn {
                    Ok(conn) => {
                        conn.luanch_client().into_actor(cli).map(|ws_cli, cli, ctx| {
                            
                        }).wait(ctx);
                    }
                    Err(e) => {
                        log::error!("Error: {:?}", e);
                    }
                }
            })
            .wait(ctx);
    }
}