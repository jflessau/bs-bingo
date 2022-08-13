create trigger players_insert
after
insert
    on bingo.players for each row execute procedure game_update_notification('players_update');
