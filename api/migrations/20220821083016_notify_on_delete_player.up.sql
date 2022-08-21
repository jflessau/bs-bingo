create trigger players_delete
after
    delete on bingo.players for each row execute procedure game_update_notification('players_update');

create or replace function game_update_notification ()
 returns trigger
 language plpgsql
as $$
declare
  channel text := tg_argv[0];
begin
  perform (
     with payload(game_id) as
     (
       select coalesce(new.game_id, old.game_id)
     )
     select pg_notify(channel, row_to_json(payload)::text)
       from payload
  );
  return null;
end;
$$;
