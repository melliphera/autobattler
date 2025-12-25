extends Node2D

class_name BattleHandler

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	pass # Replace with function body.


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	var ticks_passed = int(delta*20.0)
	# hook into BattleState, get events back 
	pass

func start_fight(instance: int) -> void:
	# get all units
	# convert into (TemplateID, BattlePosition)
	# feed to Rust alongside instance - used on rust end to generate enemies.
	# return event stream
	pass
