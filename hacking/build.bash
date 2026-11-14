#!/usr/bin/bash

if [ -e sandbox ]
then
	echo 'Character directory 'sandbox' alrfeady exists, exiting...'
	exit 1
fi

cargo run character new 'Character, The Sandbox'
mv 'Character, The Sandbox' sandbox

character_dir=sandbox
character_file=$character_dir/character.yaml
status_file=$character_dir/status.yaml
notes_file=$character_dir/notes.yaml

character_sets=(
	'.name="Character, The Sandbox"'
	'.class="Wizard"'
	'.sub_class="School of Evocation"'
	'.race="Human"'
	.level=3
	.armor_class=10
	.max_hp=11
	.speed=30

	'.abilities[0].name="strength"'
	.abilities[0].score=2
	.abilities[0].proficiencies=[]

	'.abilities[1].name="intelligence"'
	.abilities[1].score=18
	.abilities[1].proficiencies=[]

	'.abilities[2].name="dexterity"'
	.abilities[2].score=12
	.abilities[2].proficiencies=[]

	'.abilities[3].name="wisdom"'
	.abilities[3].score=11
	.abilities[3].proficiencies=[]

	'.abilities[4].name="constitution"'
	.abilities[4].score=10
	.abilities[4].proficiencies=[]

	'.abilities[5].name="charisma"'
	.abilities[5].score=8
	.abilities[5].proficiencies=[]

	.spell_slots[0]=4
	.spell_slots[1]=2

	'.spells[0].name="Blade Ward"'
	.spells[0].level=0
	'.spells[0].casting_time="1 action"'
	'.spells[0].range="Self"'
	'.spells[0].components="V, S"'
	'.spells[0].duration="1 round"'
	'.spells[0].description="You extend your hand and trace a sigil of warding in the air. Until the end of your next turn, you have resistance against bludgeoning, piercing, and slashing damage dealt by weapon attacks."'

	'.spells[1].name="Fire Bolt"'
	.spells[1].level=0
	'.spells[1].casting_time="1 action"'
	'.spells[1].range="120 feet"'
	'.spells[1].components="V, S"'
	'.spells[1].duration="Instantaneous"'
	'.spells[1].description="You hurl a mote of fire at a creature or object within range. Make a ranged spell attack against the target. On a hit, the target takes 1d10 fire damage. A flammable object hit by this spell ignites if it isn't being worn or carried."'
	'.spells[1].at_higher_levels="This spell's damage increases by 1d10 when you reach 5th level (2d10), 11th level (3d10), and 17th level (4d10)."'

	'.spells[2].name="Mold Earth"'
	.spells[2].level=0
	'.spells[2].casting_time="1 action"'
	'.spells[2].range="30 feet"'
	'.spells[2].components="S"'
	'.spells[2].duration="Instantaneous or 1 hour"'
	'.spells[2].description="You choose a portion of dirt or stone that you can see within range and that fits within a 5-foot cube. You manipulate it in one of the following ways:"'

	'.spells[3].name="Absorb Elements"'
	.spells[3].level=1
	'.spells[3].casting_time="1 reaction, which you take when you take acid, cold, fire, lightning, or thunder damage"'
	'.spells[3].range="Self"'
	'.spells[3].components="S"'
	'.spells[3].duration="1 round"'
	'.spells[3].description="The spell captures some of the incoming energy, lessening its effect on you and storing it for your next melee attack. You have resistance to the triggering damage type until the start of your next turn. Also, the first time you hit with a melee attack on your next turn, the target takes an extra 1d6 damage of the triggering type, and the spell ends."'
	'.spells[3].at_higher_levels="When you cast this spell using a spell slot of 2nd level or higher, the extra damage increases by 1d6 for each slot level above 1st."'

	'.spells[4].name="Acid Stream (UA)"'
	.spells[4].level=1
	'.spells[4].casting_time="1 action"'
	'.spells[4].range="Self (30-foot line)"'
	'.spells[4].components="V, S, M (a bit of rotten food)"'
	'.spells[4].duration="Concentration, up to 1 minute"'
	'.spells[4].description="A stream of acid emanates from you in a line 30 feet long and 5 feet wide in a direction you choose. Each creature in the line must succeed on a Dexterity saving throw or be covered in acid for the spell'"'"'s duration or until a creature uses its action to scrape or wash the acid off itself or another creature. A creature covered in the acid takes 3d4 acid damage at the start of each of its turns."'
	'.spells[4].at_higher_levels="When you cast this spell using a spell slot of 2nd level or higher, the damage increases by 1d4 for each slot level above 1st."'

	'.spells[5].name="Shield"'
	.spells[5].level=1
	'.spells[5].casting_time="1 reaction, which you take when you are hit by an attack or targeted by the "'
	'.spells[5].range="Self"'
	'.spells[5].components="V, S"'
	'.spells[5].duration="1 round"'
	'.spells[5].description=""'

	'.spells[6].name="Thunderwave"'
	.spells[6].level=1
	'.spells[6].casting_time="1 action"'
	'.spells[6].range="Self (15-foot cube)"'
	'.spells[6].components="V, S"'
	'.spells[6].duration="Instantaneous"'
	'.spells[6].description="A wave of thunderous force sweeps out from you. Each creature in a 15-foot cube originating from you must make a Constitution saving throw. On a failed save, a creature takes 2d8 thunder damage and is pushed 10 feet away from you. On a successful save, the creature takes half as much damage and is not pushed."'

	'.spells[7].name="Misty Step"'
	.spells[7].level=2
	'.spells[7].casting_time="1 bonus action"'
	'.spells[7].range="Self"'
	'.spells[7].components="V"'
	'.spells[7].duration="Instantaneous"'
	'.spells[7].description="Briefly surrounded by silvery mist, you teleport up to 30 feet to an unoccupied space that you can see."'

	'.spells[8].name="Shatter"'
	.spells[8].level=2
	'.spells[8].casting_time="1 action"'
	'.spells[8].range="60 feet"'
	'.spells[8].components="V, S, M (a chip of mica)"'
	'.spells[8].duration="Instantaneous"'
	'.spells[8].description="A sudden loud ringing noise, painfully intense, erupts from a point of your choice within range. Each creature in a 10-foot-radius sphere centered on that point must make a Constitution saving throw. A creature takes 3d8 thunder damage on a failed save, or half as much damage on a successful one. A creature made of inorganic material such as stone, crystal, or metal has disadvantage on this saving throw."'

	'.languages=["Common", "Orcish"]'

	'.proficiences=["Arcana", "Investigation"]'
)

character_query=$(IFS='|' ; echo "${character_sets[*]}")

yq -i "$character_query" $character_file

status_sets=(
	'.items[0].name="Quarterstaff"'
	'.items[0].count=0'
	'.items[0].description="1 d6 bludgeoning"'

	'.items[1].name="Bay Leaf"'
	'.items[1].count=20'
	'.items[1].description=""'
)

status_query=$(IFS='|' ; echo "${status_sets[*]}")

yq -i "$status_query" $status_file