--[=[
Target Practice Mission Framework for Hedgewars

This is a simple library intended to make setting up simple training missions a trivial
task requiring just. The library has been created to reduce redundancy in Lua scripts.

The training framework generates complete and fully usable training missions by just
one function call.

The missions generated by this script are all the same:
- The player will get a team with a single hedgehog.
- The team gets a single predefined weapon infinitely times.
- A fixed sequence of targets will spawn at predefined positions.
- When a target has been destroyed, the next target of the target sequence appears
- The mission ends successfully when all targets have been destroyed
- The mission ends unsuccessfully when the time runs out or the hedgehog dies
- When the mission ends, a score is awarded, based on the performance (hit targets,
  accuracy and remaining time) of the hedgehog. When not all targets are hit, there
  will be no accuracy and time bonuses.

To use this library, you first have to load it and to call TrainingMission once with
the appropriate parameters. Really, that’s all!
See the comment of TrainingMission for a specification of all parameters.

Below is a template for your convenience, you just have to fill in the fields and delete
optional arguments you don’t want.
----- snip -----
HedgewarsScriptLoad("/Scripts/Training.lua")
params = {
	missionTitle = ,
	map = ,
	theme = ,
	time = ,
	ammoType = ,
	gearType = ,
	secondaryGearType = ,
	targets = {
		{ x = , y = },
		{ x = , y = },
		-- etc.
	},

	wind = ,
	solidLand = ,
	artillery = ,
	clanColor = ,
	goalText = ,
	shootText =
}
TargetPracticeMission(params)
----- snip -----
]=]

HedgewarsScriptLoad("/Scripts/Utils.lua")
HedgewarsScriptLoad("/Scripts/Locale.lua")

local player = nil
local scored = 0
local shots = 0
local end_timer = 1000
local game_lost = false
local time_goal = 0
local total_targets
local targets
local target_radar = false
local next_target_circle = nil
local gearsInGameCount = 0
local gearsInGame = {}

--[[
TrainingMission(params)

This function sets up the *entire* training mission and needs one argument: params.
The argument “params” is a table containing fields which describe the training mission.
	mandatory fields:
	- missionTitle:	the name of the mission
	- map:		the name map to be used
	- theme:	the name of the theme (does not need to be a standalone theme)
	- time:		the time limit in milliseconds
	- ammoType:	the ammo type of the weapon to be used
	- gearType:	the gear type of the gear which is fired (used to count shots and re-center camera)
	- targets:	The coordinates of where the targets will be spawned.
			It is a table containing tables containing coordinates of format
			{ x=value, y=value }. The targets will be spawned in the same
			order as specified the coordinate tables appear. Example:
				targets = {
					{ x = 324, y = 43 },
					{ x = 123, y = 56 },
					{ x = 6, y = 0 },
				}
			There must be at least 1 target.

	optional fields:
	- wind:		the initial wind (-100 to 100) (default: 0 (no wind))
	- solidLand:	weather the terrain is indestructible (default: false)
	- artillery:	if true, the hog can’t move (default: false)
	- secGearType:	cluster of projectile gear (if present) (used to re-center camera)
	- clanColor:	color of the (only) clan (default: -1, default first clan color)
	- goalText:	A short string explaining the goal of the mission
			(default: "Destroy all targets within the time!")
	- shootText:	A string which says how many times the player shot, “%d” is replaced
			by the number of shots. (default: "You have shot %d times.")
	- useRadar	Whether to use target radar (small circles that mark the position
			of the next target). (default: true). Note: Still needs to be unlocked.
	- radarTint:	RGBA color of the target radar  (default: 0x8080FFFF). Use this field
			if the target radar would be hard to see against the background.
]]


local getTargetsScore = function()
	return scored * math.ceil(6000/#targets)
end

function TargetPracticeMission(params)
	if params.goalText == nil then params.goalText = loc("Eliminate all targets before your time runs out.|You have unlimited ammo for this mission.") end
	if params.shootText == nil then params.shootText = loc("You have shot %d times.") end
	if params.clanColor == nil then params.clanColor = -1 end
	if params.wind == nil then params.wind = 0 end
	if params.radarTint == nil then params.radarTint = 0x8080FFFF end
	if params.useRadar == nil then params.useRadar = true end

	local solid, artillery
	if params.solidLand == true then solid = gfSolidLand else solid = 0 end
	if params.artillery == true then artillery = gfArtillery else artillery = 0 end

	targets = params.targets

	total_targets = #targets

	_G.onAmmoStoreInit = function()
		SetAmmo(params.ammoType, 9, 0, 0, 0)
	end

	_G.onGameInit = function()
		Seed = 1
		ClearGameFlags()
		local attackMode
		if (params.ammoType == amBee) then
			attackMode = gfInfAttack
		else
			attackMode = gfMultiWeapon
		end
		EnableGameFlags(gfDisableWind, attackMode, gfOneClanMode, solid, artillery)
		TurnTime = params.time
		Map = params.map
		Theme = params.theme
		Goals = params.goalText
		CaseFreq = 0
		MinesNum = 0
		Explosives = 0
		-- Disable Sudden Death
		WaterRise = 0
		HealthDecrease = 0

		SetWind(params.wind)

		AddMissionTeam(params.clanColor)

		player = AddMissionHog(1)
		SetGearPosition(player, params.hog_x, params.hog_y)

		local won = GetMissionVar("Won")
		-- Unlock the target radar when the player has completed
		-- the target practice before (any score).
		-- Target radar might be disabled by config, however.
		if won == "true" and params.useRadar == true then
			target_radar = true
		end

	end

	_G.onGameStart = function()
		SendHealthStatsOff()
		local recordInfo = getReadableChallengeRecord("Highscore")
		ShowMission(params.missionTitle, loc("Aiming practice"), params.goalText .. "|" .. recordInfo, -params.ammoType, 5000)
		SetTeamLabel(GetHogTeamName(player), "0")
		spawnTarget()
	end

	_G.onNewTurn = function()
		SetWeapon(params.ammoType)
	end

	_G.spawnTarget = function()
		-- Spawn next target
		local gear = AddGear(0, 0, gtTarget, 0, 0, 0, 0)

		local x = targets[scored+1].x
		local y = targets[scored+1].y

		SetGearPosition(gear, x, y)

		-- Target radar: Highlight position of the upcoming target.
		-- This must be unlocked by the player first.
		if target_radar then
			if (not next_target_circle) and targets[scored+2] then
				next_target_circle = AddVisualGear(0,0,vgtCircle,90,true)
			end
			if targets[scored+2] then
				SetVisualGearValues(next_target_circle, targets[scored+2].x, targets[scored+2].y, 205, 255, 1, 20, nil, nil, 3, params.radarTint)
			elseif next_target_circle then
				DeleteVisualGear(next_target_circle)
				next_target_circle = nil
			end
		end

		return gear
	end

	_G.onGameTick20 = function()
		if TurnTimeLeft < 40 and TurnTimeLeft > 0 and scored < total_targets and game_lost == false then
			game_lost = true
			AddCaption(loc("Time’s up!"), capcolDefault, capgrpGameState)
			SetHealth(player, 0)
			time_goal = 1
		end

		if band(GetState(player), gstDrowning) == gstDrowning and game_lost == false and scored < total_targets then
			game_lost = true
			time_goal = 1
		end

		if scored == total_targets  or game_lost then
			if end_timer == 0 then
				generateStats()
				EndGame()
			end
			end_timer = end_timer - 20
		end

		for gear, _ in pairs(gearsInGame) do
			if band(GetState(gear), gstDrowning) ~= 0 then
				-- Re-center camera on hog if projectile gears drown
				gearsInGame[gear] = nil
				gearsInGameCount = gearsInGameCount - 1
				if gearsInGameCount == 0 and GetHealth(CurrentHedgehog) then
					FollowGear(CurrentHedgehog)
				end
			end
		end
	end

	_G.onGearAdd = function(gear)
		if GetGearType(gear) == params.gearType then
			shots = shots + 1
		end
		if GetGearType(gear) == params.gearType or (params.secGearType and GetGearType(gear) == params.secGearType) then
			gearsInGameCount = gearsInGameCount + 1
			gearsInGame[gear] = true
		end
	end

	_G.onGearDamage = function(gear, damage)
		if GetGearType(gear) == gtTarget then
			scored = scored + 1
			SetTeamLabel(GetHogTeamName(player), tostring(getTargetsScore()))
			if scored < total_targets then
				AddCaption(string.format(loc("Targets left: %d"), (total_targets-scored)), capcolDefault, capgrpMessage)
				spawnTarget()
			else
				if not game_lost then
					SaveMissionVar("Won", "true")
					AddCaption(loc("You have destroyed all targets!"), capcolDefault, capgrpGameState)
					ShowMission(params.missionTitle, loc("Aiming practice"), loc("Congratulations! You have destroyed all targets within the time."), 0, 0)
					PlaySound(sndVictory, player)
					SetEffect(player, heInvulnerable, 1)
					SetState(player, bor(GetState(player), gstWinner))
					time_goal = TurnTimeLeft
					-- Disable control
					SetInputMask(0)
					AddAmmo(player, params.ammoType, 0)
					SetTurnTimePaused(true)
				end
			end
		end

		if GetGearType(gear) == gtHedgehog then
			if not game_lost then
				game_lost = true

				SetHealth(player, 0)
				time_goal = 1
			end
		end
	end

	_G.onGearDelete = function(gear)
		if GetGearType(gear) == gtTarget and band(GetState(gear), gstDrowning) ~= 0 then
			AddCaption(loc("You lost your target, try again!"), capcolDefault, capgrpGameState)
			local newTarget = spawnTarget()
			local x, y = GetGearPosition(newTarget)
			local success = PlaceSprite(x, y + 24, sprAmGirder, 0, 0xFFFFFFFF, false, false, false)
			if not success then
				WriteLnToConsole("ERROR: Failed to spawn girder under respawned target!")
			end
		elseif gearsInGame[gear] then
			gearsInGame[gear] = nil
			gearsInGameCount = gearsInGameCount - 1
			if gearsInGameCount == 0 and GetHealth(CurrentHedgehog) then
				-- Re-center camera to hog after all projectile gears were destroyed
				FollowGear(CurrentHedgehog)
			end
		end
	end

	_G.generateStats = function()
		local accuracy, accuracy_int
		if(shots > 0) then
			accuracy = (scored/shots)*100
			accuracy_int = div(scored*100, shots)
		end
		local end_score_targets = getTargetsScore()
		local end_score_overall
		if not game_lost then
			local end_score_time = math.ceil(time_goal/(params.time/6000))
			local end_score_accuracy = 0
			if(shots > 0) then
				end_score_accuracy = math.ceil(accuracy * 60)
			end
			end_score_overall = end_score_time + end_score_targets + end_score_accuracy
			SetTeamLabel(GetHogTeamName(player), tostring(end_score_overall))

			SendStat(siGameResult, loc("You have finished the target practice!"))

			SendStat(siCustomAchievement, string.format(loc("You have destroyed %d of %d targets (+%d points)."), scored, total_targets, end_score_targets))
			SendStat(siCustomAchievement, string.format(params.shootText, shots))
			if(shots > 0) then
				SendStat(siCustomAchievement, string.format(loc("Your accuracy was %.1f%% (+%d points)."), accuracy, end_score_accuracy))
			end
			SendStat(siCustomAchievement, string.format(loc("You had %.1fs remaining on the clock (+%d points)."), (time_goal/1000), end_score_time))
			if (not target_radar) and (#targets > 1) and (params.useRadar == true) then
				SendStat(siCustomAchievement, loc("You have unlocked the target radar!"))
			end

			if(shots > 0) then
				updateChallengeRecord("AccuracyRecord", accuracy_int)
			end
		else
			SendStat(siGameResult, loc("Challenge over!"))

			SendStat(siCustomAchievement, string.format(loc("You have destroyed %d of %d targets (+%d points)."), scored, total_targets, end_score_targets))
			SendStat(siCustomAchievement, string.format(params.shootText, shots))
			if(shots > 0) then
				SendStat(siCustomAchievement, string.format(loc("Your accuracy was %.1f%%."), accuracy))
			end
			end_score_overall = end_score_targets
		end
		SendStat(siPointType, "!POINTS")
		SendStat(siPlayerKills, tostring(end_score_overall), GetHogTeamName(player))
		-- Update highscore
		updateChallengeRecord("Highscore", end_score_overall)
	end
end
