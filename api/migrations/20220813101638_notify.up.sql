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

create trigger fields_update
         after update
            on bingo.fields
      for each row
       execute procedure game_update_notification('fields_update');

create trigger players_update
         after update
            on bingo.players
      for each row
       execute procedure game_update_notification('players_update');
