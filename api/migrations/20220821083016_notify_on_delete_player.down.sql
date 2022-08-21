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
       select new.game_id
     )
     select pg_notify(channel, row_to_json(payload)::text)
       from payload
  );
  return null;
end;
$$;

drop trigger players_delete on bingo.players;
