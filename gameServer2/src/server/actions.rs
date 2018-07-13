use std::{
    io, io::Write,
    iter::once,
    mem::replace
};
use super::{
    server::HWServer,
    room::GameInfo,
    client::HWClient,
    coretypes::{ClientId, RoomId, GameCfg, VoteType},
    room::HWRoom,
    handlers
};
use protocol::messages::{
    HWProtocolMessage,
    HWServerMessage,
    HWServerMessage::*,
    server_chat
};
use utils::to_engine_msg;
use rand::{thread_rng, Rng, distributions::Uniform};

pub enum Destination {
    ToId(ClientId),
    ToSelf,
    ToAll {
        room_id: Option<RoomId>,
        protocol: Option<u16>,
        skip_self: bool
    }
}

pub struct PendingMessage {
    pub destination: Destination,
    pub message: HWServerMessage
}

impl PendingMessage {
    pub fn send(message: HWServerMessage, client_id: ClientId) -> PendingMessage {
        PendingMessage{ destination: Destination::ToId(client_id), message}
    }

    pub fn send_self(message: HWServerMessage) -> PendingMessage {
        PendingMessage{ destination: Destination::ToSelf, message }
    }

    pub fn send_all(message: HWServerMessage) -> PendingMessage {
        let destination = Destination::ToAll {
            room_id: None,
            protocol: None,
            skip_self: false,
        };
        PendingMessage{ destination, message }
    }

    pub fn in_room(mut self, clients_room_id: RoomId) -> PendingMessage {
        if let Destination::ToAll {ref mut room_id, ..} = self.destination {
            *room_id = Some(clients_room_id)
        }
        self
    }

    pub fn with_protocol(mut self, protocol_number: u16) -> PendingMessage {
        if let Destination::ToAll {ref mut protocol, ..} = self.destination {
            *protocol = Some(protocol_number)
        }
        self
    }

    pub fn but_self(mut self) -> PendingMessage {
        if let Destination::ToAll {ref mut skip_self, ..} = self.destination {
            *skip_self = true
        }
        self
    }

    pub fn action(self) -> Action { Send(self) }
}

impl Into<Action> for PendingMessage {
    fn into(self) -> Action { self.action() }
}

impl HWServerMessage {
    pub fn send(self, client_id: ClientId) -> PendingMessage { PendingMessage::send(self, client_id) }
    pub fn send_self(self) -> PendingMessage { PendingMessage::send_self(self) }
    pub fn send_all(self) -> PendingMessage { PendingMessage::send_all(self) }
}

pub enum Action {
    Send(PendingMessage),
    RemoveClient,
    ByeClient(String),
    ReactProtocolMessage(HWProtocolMessage),
    CheckRegistered,
    JoinLobby,
    AddRoom(String, Option<String>),
    RemoveRoom(RoomId),
    MoveToRoom(RoomId),
    MoveToLobby(String),
    ChangeMaster(RoomId, Option<ClientId>),
    RemoveTeam(String),
    RemoveClientTeams,
    SendRoomUpdate(Option<String>),
    StartRoomGame(RoomId),
    SendTeamRemovalMessage(String),
    FinishRoomGame(RoomId),
    SendRoomData{to: ClientId, teams: bool, config: bool, flags: bool},
    AddVote{vote: bool, is_forced: bool},
    ApplyVoting(VoteType, RoomId),
    Warn(String),
    ProtocolError(String)
}

use self::Action::*;

pub fn run_action(server: &mut HWServer, client_id: usize, action: Action) {
    match action {
        Send(msg) => server.send(client_id, msg.destination, msg.message),
        ByeClient(msg) => {
            let room_id;
            let nick;
            {
                let c = &server.clients[client_id];
                room_id = c.room_id;
                nick = c.nick.clone();
            }

            room_id.map (|id| {
                if id != server.lobby_id {
                    server.react(client_id, vec![
                        MoveToLobby(format!("quit: {}", msg.clone()))]);
                }
            });

            server.react(client_id, vec![
                LobbyLeft(nick, msg.clone()).send_all().action(),
                Bye(msg).send_self().action(),
                RemoveClient]);
        },
        RemoveClient => {
            server.removed_clients.push(client_id);
            if server.clients.contains(client_id) {
                server.clients.remove(client_id);
            }
        },
        ReactProtocolMessage(msg) =>
            handlers::handle(server, client_id, msg),
        CheckRegistered =>
            if server.clients[client_id].protocol_number > 0 && server.clients[client_id].nick != "" {
                server.react(client_id, vec![
                    JoinLobby,
                    ]);
            },
        JoinLobby => {
            server.clients[client_id].room_id = Some(server.lobby_id);

            let joined_msg;
            {
                let mut lobby_nicks = Vec::new();
                for (_, c) in server.clients.iter() {
                    if c.room_id.is_some() {
                        lobby_nicks.push(c.nick.clone());
                    }
                }
                joined_msg = LobbyJoined(lobby_nicks);
            }
            let everyone_msg = LobbyJoined(vec![server.clients[client_id].nick.clone()]);
            let flags_msg = ClientFlags(
                "+i".to_string(),
                server.clients.iter()
                    .filter(|(_, c)| c.room_id.is_some())
                    .map(|(_, c)| c.nick.clone())
                    .collect());
            let server_msg = ServerMessage("\u{1f994} is watching".to_string());
            let rooms_msg = Rooms(server.rooms.iter()
                .filter(|(id, _)| *id != server.lobby_id)
                .flat_map(|(_, r)|
                    r.info(r.master_id.map(|id| &server.clients[id])))
                .collect());
            server.react(client_id, vec![
                everyone_msg.send_all().but_self().action(),
                joined_msg.send_self().action(),
                flags_msg.send_self().action(),
                server_msg.send_self().action(),
                rooms_msg.send_self().action(),
                ]);
        },
        AddRoom(name, password) => {
            let room_id = server.add_room();;
            let actions = {
                let r = &mut server.rooms[room_id];
                let c = &mut server.clients[client_id];
                r.master_id = Some(c.id);
                r.name = name;
                r.password = password;
                r.protocol_number = c.protocol_number;

                vec![
                    RoomAdd(r.info(Some(&c))).send_all()
                        .with_protocol(r.protocol_number).action(),
                    MoveToRoom(room_id)]
            };
            server.react(client_id, actions);
        },
        RemoveRoom(room_id) => {
            let actions = {
                let r = &mut server.rooms[room_id];
                vec![RoomRemove(r.name.clone()).send_all()
                        .with_protocol(r.protocol_number).action()]
            };
            server.rooms.remove(room_id);
            server.react(client_id, actions);
        }
        MoveToRoom(room_id) => {
            let actions = {
                let r = &mut server.rooms[room_id];
                let c = &mut server.clients[client_id];
                r.players_number += 1;
                c.room_id = Some(room_id);

                let is_master = r.master_id == Some(c.id);
                c.set_is_master(is_master);
                c.set_is_ready(is_master);
                c.set_is_joined_mid_game(false);

                if is_master {
                    r.ready_players_number += 1;
                }

                let mut v = vec![
                    RoomJoined(vec![c.nick.clone()]).send_all().in_room(room_id).action(),
                    ClientFlags("+i".to_string(), vec![c.nick.clone()]).send_all().action(),
                    SendRoomUpdate(None)];
                if !r.greeting.is_empty() {
                    v.push(ChatMsg {nick: "[greeting]".to_string(), msg: r.greeting.clone()}
                        .send_self().action());
                }
                if !c.is_master() {
                    let team_names: Vec<_>;
                    if let Some(ref mut info) = r.game_info {
                        c.set_is_in_game(true);
                        c.set_is_joined_mid_game(true);

                        {
                            let teams = info.client_teams(c.id);
                            c.teams_in_game = teams.clone().count() as u8;
                            c.clan = teams.clone().next().map(|t| t.color);
                            team_names = teams.map(|t| t.name.clone()).collect();
                        }

                        if !team_names.is_empty() {
                            info.left_teams.retain(|name|
                                !team_names.contains(&name));
                            info.teams_in_game += team_names.len() as u8;
                            r.teams = info.teams_at_start.iter()
                                .filter(|(_, t)| !team_names.contains(&t.name))
                                .cloned().collect();
                        }
                    } else {
                        team_names = Vec::new();
                    }

                    v.push(SendRoomData{ to: client_id, teams: true, config: true, flags: true});

                    if let Some(ref info) = r.game_info {
                        v.push(RunGame.send_self().action());
                        v.push(ClientFlags("+g".to_string(), vec![c.nick.clone()])
                            .send_all().in_room(r.id).action());
                        v.push(ForwardEngineMessage(
                            vec![to_engine_msg("e$spectate 1".bytes())])
                            .send_self().action());
                        v.push(ForwardEngineMessage(info.msg_log.clone())
                            .send_self().action());

                        for name in team_names.iter() {
                            v.push(ForwardEngineMessage(
                                vec![to_engine_msg(once(b'G').chain(name.bytes()))])
                                .send_all().in_room(r.id).action());
                        }
                        if info.is_paused {
                            v.push(ForwardEngineMessage(vec![to_engine_msg(once(b'I'))])
                                .send_all().in_room(r.id).action())
                        }
                    }
                }
                v
            };
            server.react(client_id, actions);
        }
        SendRoomData {to, teams, config, flags} => {
            let mut actions = Vec::new();
            let room_id = server.clients[client_id].room_id;
            if let Some(r) = room_id.and_then(|id| server.rooms.get(id)) {
                if config {
                    actions.push(ConfigEntry("FULLMAPCONFIG".to_string(), r.map_config())
                        .send(to).action());
                    for cfg in r.game_config().into_iter() {
                        actions.push(cfg.to_server_msg().send(to).action());
                    }
                }
                if teams {
                    let current_teams = match r.game_info {
                        Some(ref info) => &info.teams_at_start,
                        None => &r.teams
                    };
                    for (owner_id, team) in current_teams.iter() {
                        actions.push(TeamAdd(HWRoom::team_info(&server.clients[*owner_id], &team))
                            .send(to).action());
                        actions.push(TeamColor(team.name.clone(), team.color)
                            .send(to).action());
                        actions.push(HedgehogsNumber(team.name.clone(), team.hedgehogs_number)
                            .send(to).action());
                    }
                }
                if flags {
                    if let Some(id) = r.master_id {
                        actions.push(ClientFlags("+h".to_string(), vec![server.clients[id].nick.clone()])
                            .send(to).action());
                    }
                    let nicks: Vec<_> = server.clients.iter()
                        .filter(|(_, c)| c.room_id == Some(r.id) && c.is_ready())
                        .map(|(_, c)| c.nick.clone()).collect();
                    if !nicks.is_empty() {
                        actions.push(ClientFlags("+r".to_string(), nicks)
                            .send(to).action());
                    }
                }
            }
            server.react(client_id, actions);
        }
        AddVote{vote, is_forced} => {
            let mut actions = Vec::new();
            if let (c, Some(r)) = server.client_and_room(client_id) {
                let mut result = None;
                if let Some(ref mut voting) = r.voting {
                    if is_forced || voting.votes.iter().find(|(id, _)| client_id == *id).is_none() {
                        actions.push(server_chat("Your vote has been counted.").send_self().action());
                        voting.votes.push((client_id, vote));
                        let i = voting.votes.iter();
                        let pro = i.clone().filter(|(_, v)| *v).count();
                        let contra = i.filter(|(_, v)| !*v).count();
                        let success_quota = voting.voters.len() / 2 + 1;
                        if is_forced && vote || pro >= success_quota {
                            result = Some(true);
                        } else if is_forced && !vote || contra > voting.voters.len() - success_quota {
                            result = Some(false);
                        }
                    } else {
                        actions.push(server_chat("You already have voted.").send_self().action());
                    }
                } else {
                    actions.push(server_chat("There's no voting going on.").send_self().action());
                }

                if let Some(res) = result {
                    actions.push(server_chat("Voting closed.")
                        .send_all().in_room(r.id).action());
                    let voting = replace(&mut r.voting, None).unwrap();
                    if res {
                        actions.push(ApplyVoting(voting.kind, r.id));
                    }
                }
            }

            server.react(client_id, actions);
        }
        ApplyVoting(kind, room_id) => {
            let mut actions = Vec::new();
            let mut id = client_id;
            match kind {
                VoteType::Kick(nick) => {
                    if let Some(c) = server.find_client(&nick) {
                        if c.room_id == Some(room_id) {
                            id = c.id;
                            actions.push(Kicked.send_self().action());
                            actions.push(MoveToLobby("kicked".to_string()));
                        }
                    }
                },
                VoteType::Map(_) => {
                    unimplemented!();
                },
                VoteType::Pause => {
                    if let Some(ref mut info) = server.rooms[room_id].game_info {
                        info.is_paused = !info.is_paused;
                        actions.push(server_chat("Pause toggled.")
                            .send_all().in_room(room_id).action());
                        actions.push(ForwardEngineMessage(vec![to_engine_msg(once(b'I'))])
                            .send_all().in_room(room_id).action());
                    }
                },
                VoteType::NewSeed => {
                    let seed = thread_rng().gen_range(0, 1_000_000_000).to_string();
                    let cfg = GameCfg::Seed(seed);
                    actions.push(cfg.to_server_msg().send_all().in_room(room_id).action());
                    server.rooms[room_id].set_config(cfg);
                },
                VoteType::HedgehogsPerTeam(number) => {
                    let r = &mut server.rooms[room_id];
                    let nicks = r.set_hedgehogs_number(number);
                    actions.extend(nicks.into_iter().map(|n|
                        HedgehogsNumber(n, number).send_all().in_room(room_id).action()
                    ));
                },
            }
            server.react(id, actions);
        }
        MoveToLobby(msg) => {
            let mut actions = Vec::new();
            let lobby_id = server.lobby_id;
            if let (c, Some(r)) = server.client_and_room(client_id) {
                r.players_number -= 1;
                if c.is_ready() && r.ready_players_number > 0 {
                    r.ready_players_number -= 1;
                }
                if c.is_master() && (r.players_number > 0 || r.is_fixed) {
                    actions.push(ChangeMaster(r.id, None));
                }
                actions.push(ClientFlags("-i".to_string(), vec![c.nick.clone()])
                    .send_all().action());
            }
            server.react(client_id, actions);
            actions = Vec::new();

            if let (c, Some(r)) = server.client_and_room(client_id) {
                c.room_id = Some(lobby_id);
                if r.players_number == 0 && !r.is_fixed {
                    actions.push(RemoveRoom(r.id));
                } else {
                    actions.push(RemoveClientTeams);
                    actions.push(RoomLeft(c.nick.clone(), msg)
                        .send_all().in_room(r.id).but_self().action());
                    actions.push(SendRoomUpdate(Some(r.name.clone())));
                }
            }
            server.react(client_id, actions)
        }
        ChangeMaster(room_id, new_id) => {
            let mut actions = Vec::new();
            let room_client_ids = server.room_clients(room_id);
            let new_id = if server.room(client_id).map(|r| r.is_fixed).unwrap_or(false) {
                new_id
            } else {
                new_id.or_else(||
                    room_client_ids.iter().find(|id| **id != client_id).map(|id| *id))
            };
            let new_nick = new_id.map(|id| server.clients[id].nick.clone());

            if let (c, Some(r)) = server.client_and_room(client_id) {
                match r.master_id {
                    Some(id) if id == c.id => {
                        c.set_is_master(false);
                        r.master_id = None;
                        actions.push(ClientFlags("-h".to_string(), vec![c.nick.clone()])
                            .send_all().in_room(r.id).action());
                    }
                    Some(_) => unreachable!(),
                    None => {}
                }
                r.master_id = new_id;
                if let Some(nick) = new_nick {
                    actions.push(ClientFlags("+h".to_string(), vec![nick])
                        .send_all().in_room(r.id).action());
                }
            }
            new_id.map(|id| server.clients[id].set_is_master(true));
            server.react(client_id, actions);
        }
        RemoveTeam(name) => {
            let mut actions = Vec::new();
            if let (c, Some(r)) = server.client_and_room(client_id) {
                r.remove_team(&name);
                if let Some(ref mut info) = r.game_info {
                    info.left_teams.push(name.clone());
                }
                actions.push(TeamRemove(name.clone()).send_all().in_room(r.id).action());
                actions.push(SendRoomUpdate(None));
                if r.game_info.is_some() && c.is_in_game() {
                    actions.push(SendTeamRemovalMessage(name));
                }
            }
            server.react(client_id, actions);
        },
        RemoveClientTeams => {
            let actions = if let (c, Some(r)) = server.client_and_room(client_id) {
                r.client_teams(c.id).map(|t| RemoveTeam(t.name.clone())).collect()
            } else {
                Vec::new()
            };
            server.react(client_id, actions);
        }
        SendRoomUpdate(old_name) => {
            let actions = if let (c, Some(r)) = server.client_and_room(client_id) {
                let name = old_name.unwrap_or_else(|| r.name.clone());
                vec![RoomUpdated(name, r.info(Some(&c)))
                    .send_all().with_protocol(r.protocol_number).action()]
            } else {
                Vec::new()
            };
            server.react(client_id, actions);
        },
        StartRoomGame(room_id) => {
            let actions = {
                let (room_clients, room_nicks): (Vec<_>, Vec<_>) = server.clients.iter()
                    .map(|(id, c)| (id, c.nick.clone())).unzip();
                let room = &mut server.rooms[room_id];

                if !room.has_multiple_clans() {
                    vec![Warn("The game can't be started with less than two clans!".to_string())]
                } else if room.game_info.is_some() {
                    vec![Warn("The game is already in progress".to_string())]
                } else {
                    room.start_round();
                    for id in room_clients {
                        let c = &mut server.clients[id];
                        c.set_is_in_game(false);
                        c.team_indices = room.client_team_indices(c.id);
                    }
                    vec![RunGame.send_all().in_room(room.id).action(),
                         SendRoomUpdate(None),
                         ClientFlags("+g".to_string(), room_nicks)
                             .send_all().in_room(room.id).action()]
                }
            };
            server.react(client_id, actions);
        }
        SendTeamRemovalMessage(team_name) => {
            let mut actions = Vec::new();
            if let (c, Some(r)) = server.client_and_room(client_id) {
                if let Some(ref mut info) = r.game_info {
                    let msg = once(b'F').chain(team_name.bytes());
                    actions.push(ForwardEngineMessage(vec![to_engine_msg(msg)]).
                        send_all().in_room(r.id).but_self().action());
                    info.teams_in_game -= 1;
                    if info.teams_in_game == 0 {
                        actions.push(FinishRoomGame(r.id));
                    }
                    let remove_msg = to_engine_msg(once(b'F').chain(team_name.bytes()));
                    if let Some(m) = &info.sync_msg {
                        info.msg_log.push(m.clone());
                    }
                    if info.sync_msg.is_some() {
                        info.sync_msg = None
                    }
                    info.msg_log.push(remove_msg.clone());
                    actions.push(ForwardEngineMessage(vec![remove_msg])
                        .send_all().in_room(r.id).but_self().action());
                }
            }
            server.react(client_id, actions);
        }
        FinishRoomGame(room_id) => {
            let mut actions = Vec::new();
            let old_info;
            {
                let r = &mut server.rooms[room_id];
                old_info = replace(&mut r.game_info, None);
                r.game_info = None;
                r.ready_players_number = 1;
                actions.push(SendRoomUpdate(None));
                actions.push(RoundFinished.send_all().in_room(r.id).action());
            }

            if let Some(info) = old_info {
                for (_, c) in server.clients.iter() {
                    if c.room_id == Some(room_id) && c.is_joined_mid_game() {
                        actions.push(SendRoomData{
                            to: c.id, teams: false,
                            config: true, flags: false});
                        for name in info.left_teams.iter() {
                            actions.push(TeamRemove(name.clone())
                                .send(c.id).action());
                        }
                    }
                }
            }

            let nicks: Vec<_> = server.clients.iter_mut()
                .filter(|(_, c)| c.room_id == Some(room_id))
                .map(|(_, c)| {
                    let is_master = c.is_master();
                    c.set_is_ready(is_master);
                    c.set_is_joined_mid_game(false);
                    c
                }).filter_map(|c| if !c.is_master() {
                    Some(c.nick.clone())
                } else {
                    None
                }).collect();
            if !nicks.is_empty() {
                actions.push(ClientFlags("-r".to_string(), nicks)
                    .send_all().in_room(room_id).action());
            }
            server.react(client_id, actions);
        }
        Warn(msg) => {
            run_action(server, client_id, Warning(msg).send_self().action());
        }
        ProtocolError(msg) => {
            run_action(server, client_id, Error(msg).send_self().action())
        }
    }
}
