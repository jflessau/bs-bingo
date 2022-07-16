create table bingo.players (
    "user_id" uuid not null,
    game_id uuid not null,
    "username" text not null check (
        length("username") > 0
        and length("username") <= 24
    ),
    primary key ("user_id", game_id)
);
