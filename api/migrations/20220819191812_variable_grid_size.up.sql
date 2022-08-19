alter table
    bingo.games
add
    column grid_size integer not null default 25 check (grid_size >= 2);
