----------------------------------
-- THE SPECIALISTS MODE 0.7
-- by mikade
----------------------------------

-- version history
-----------------
-- version 0.1
-----------------
-- concept test

----------------
-- version 0.2
----------------
-- added gfRandomOrder to gameflags
-- removed some deprecated variables/methods
-- fixed lack of portal reset

----------------
-- version 0.3
----------------
-- added switching on start
-- removed switch from engineer weaponset

----------------
-- version 0.4
----------------
-- Attempted to:
-- fix potential switch explit
-- improve user feedback on start

----------------
-- version 0.5
----------------
-- provision for variable minetimer / demo mines set to 5000ms
-- don't autoswitch if player only has 1 hog on his team

----------------
-- version 0.6
----------------
-- for the meanwhile, don't drop any crates except health crates

----------------
-- version 0.7
----------------
-- perhogadmsdf :D :D :D :D

--------------------
--TO DO
--------------------

-- balance hog health, maybe
-- add proper gameflag checking, maybe (so that we can throw in a .cfg and let the users break everything)

HedgewarsScriptLoad("/Scripts/Locale.lua")
HedgewarsScriptLoad("/Scripts/Tracker.lua")
HedgewarsScriptLoad("/Scripts/Params.lua")

-- S=(S)oldier D=(D)emo E=(E)ngineer N=(N)inja P=(P)yro C=(C)lown H=(H)olyman[saint] X=Sniper [running out of letters, but X-out or X-hair or something]
-- default team values
local currTeamIdx = 0;
local teamRoles = 
        {
            {'S','E','N','D','X','H','P','C'},
            {'S','E','N','D','X','H','P','C'},
            {'S','E','N','D','X','H','P','C'},
            {'S','E','N','D','X','H','P','C'},
            {'S','E','N','D','X','H','P','C'},
            {'S','E','N','D','X','H','P','C'},
            {'S','E','N','D','X','H','P','C'},
            {'S','E','N','D','X','H','P','C'}
        };

local numhhs = 0
local hhs = {}

local started = false

function onParameters()
	parseParams()
	for i = 1, 8 do
        if params['t'..i] ~= nil then
            for j = 1, 8 do
                if string.len(params['t'..i]) >= j  then
                    teamRoles[i][j] = string.upper(string.sub(params['t'..i],j,j));
                end
            end
        end
    end
end

function onNewAmmoStore(groupIndex, hogIndex)

	SetAmmo(amSkip, 9, 0, 0, 0)
    groupIndex = groupIndex + 1
    hogIndex = hogIndex + 1

    if teamRoles[groupIndex][hogIndex] == 'S' then
		SetAmmo(amBazooka, 1, 0, 0, 0)
		SetAmmo(amGrenade, 1, 0, 0, 0)
		SetAmmo(amShotgun, 1, 0, 0, 0)
    elseif teamRoles[groupIndex][hogIndex] == 'E' then
		SetAmmo(amGirder, 2, 0, 0, 0)
		SetAmmo(amBlowTorch, 1, 0, 0, 0)
		SetAmmo(amPickHammer, 1, 0, 0, 0)
    elseif teamRoles[groupIndex][hogIndex] == 'N' then
		SetAmmo(amRope, 9, 0, 0, 0)
		SetAmmo(amParachute, 9, 0, 0, 0)
		SetAmmo(amFirePunch, 1, 0, 0, 0)
    elseif teamRoles[groupIndex][hogIndex] == 'D' then
		SetAmmo(amDynamite, 1, 0, 0, 0)
		SetAmmo(amMine, 1, 0, 0, 0)
		SetAmmo(amDrill, 1, 0, 0, 0)
    elseif teamRoles[groupIndex][hogIndex] == 'X' then
		SetAmmo(amSniperRifle, 1, 0, 0, 0)
		SetAmmo(amDEagle, 1, 0, 0, 0)
		SetAmmo(amPortalGun, 2, 0, 0, 0)
    elseif teamRoles[groupIndex][hogIndex] == 'H' then
		SetAmmo(amSeduction, 9, 0, 0, 0)
		SetAmmo(amResurrector, 1, 0, 0, 0)
		SetAmmo(amInvulnerable, 1, 0, 0, 0)
        SetAmmo(amLowGravity, 1, 0, 0, 0)
    elseif teamRoles[groupIndex][hogIndex] == 'P' then
		SetAmmo(amFlamethrower, 1, 0, 0, 0)
		SetAmmo(amMolotov, 1, 0, 0, 0)
		SetAmmo(amNapalm, 1, 0, 0, 0)
    elseif teamRoles[groupIndex][hogIndex] == 'C' then
		SetAmmo(amBaseballBat, 1, 0, 0, 0)
		SetAmmo(amGasBomb, 1, 0, 0, 0)
		SetAmmo(amKamikaze, 1, 0, 0, 0)
	end

end

function CreateTeam()

	currTeam = ""
	lastTeam = ""
	z = 0

	for i = 0, (numhhs-1) do

			currTeam = GetHogTeamName(hhs[i])

			if currTeam == lastTeam then
					z = z + 1
			else
					z = 1
                    currTeamIdx = currTeamIdx + 1;
			end

			if teamRoles[currTeamIdx][z] == 'S' then

					SetHogName(hhs[i],loc("Soldier"))
					SetHogHat(hhs[i], "sf_vega")
					SetHealth(hhs[i],200)

			elseif teamRoles[currTeamIdx][z] == 'E' then

					SetHogHat(hhs[i], "Glasses")
					SetHogName(hhs[i],loc("Engineer"))

			elseif teamRoles[currTeamIdx][z] == 'N' then

					SetHogName(hhs[i],loc("Ninja"))
					SetHogHat(hhs[i], "NinjaFull")
					SetHealth(hhs[i],80)

			elseif teamRoles[currTeamIdx][z] == 'D' then

					SetHogName(hhs[i],loc("Demo"))
					SetHogHat(hhs[i], "Skull")
					SetHealth(hhs[i],200)

			elseif teamRoles[currTeamIdx][z] == 'X' then

					SetHogName(hhs[i],loc("Sniper"))
					SetHogHat(hhs[i], "Sniper")
					SetHealth(hhs[i],120)

			elseif teamRoles[currTeamIdx][z] == 'H' then

					SetHogName(hhs[i],loc("Saint"))
					SetHogHat(hhs[i], "angel")
					SetHealth(hhs[i],300)

			elseif teamRoles[currTeamIdx][z] == 'P' then

					SetHogName(hhs[i],loc("Pyro"))
					SetHogHat(hhs[i], "Gasmask")
					SetHealth(hhs[i],150)

			elseif teamRoles[currTeamIdx][z] == 'C' then

					SetHogName(hhs[i],loc("Loon"))
					SetHogHat(hhs[i], "clown")
					SetHealth(hhs[i],100)

			end

			lastTeam = GetHogTeamName(hhs[i])

	end

end

function onGameInit()
	ClearGameFlags()
	EnableGameFlags(gfResetWeps, gfInfAttack, gfPlaceHog, gfPerHogAmmo, gfSwitchHog)
	HealthCaseProb = 100
end

function onGameStart()

	CreateTeam()

	ShowMission     (
                                loc("THE SPECIALISTS"),
                                loc("a Hedgewars mini-game"),

                                loc("Eliminate the enemy specialists.") .. "|" ..
                                " " .. "|" ..

                                loc("Game Modifiers: ") .. "|" ..
                                loc("Per-Hog Ammo") .. "|" ..
                                loc("Weapons Reset") .. "|" ..
                                loc("Unlimited Attacks") .. "|" ..

                                "", 4, 4000
                                )

	trackTeams()

end


function onNewTurn()

	started = true
	AddCaption(loc("Prepare yourself") .. ", " .. GetHogName(CurrentHedgehog).. "!")

end

function onGearAdd(gear)

    if GetGearType(gear) == gtHedgehog then
		hhs[numhhs] = gear
		numhhs = numhhs + 1
	elseif (GetGearType(gear) == gtMine) and (started == true) then
		SetTimer(gear,5000)
	end

	if (GetGearType(gear) == gtHedgehog) or (GetGearType(gear) == gtResurrector) then
		trackGear(gear)
	end

end

function onGearDelete(gear)
	if (GetGearType(gear) == gtHedgehog) or (GetGearType(gear) == gtResurrector) then
		trackDeletion(gear)
	end
end

