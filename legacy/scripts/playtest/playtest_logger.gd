class_name PlaytestLogger
extends Logger

# ==============================================================================
# Tactical RPG - Playtest Logger (PLAYTEST1 trail `error` events)
# Registered with OS.add_logger only while a recording runs (PlaytestRecorder.start_hooks). Engine and script
# errors and warnings are queued under a mutex (the engine may log from any thread); the recorder drains the
# queue into `error` trail events on the main thread. Plain prints (_log_message) are not recorded.
# ==============================================================================

const LEVELS: Array[String] = ["error", "warning", "script", "error"]   # Logger.ErrorType: error, warning, script, shader
const MSG_MAX: int = 300

var _mutex: Mutex = Mutex.new()
var _queue: Array[Dictionary] = []

func _log_error(function: String, file: String, line: int, code: String, rationale: String, _editor_notify: bool, error_type: int, script_backtraces: Array[ScriptBacktrace]) -> void:
	var msg: String = rationale if not rationale.is_empty() else code
	var where: String = "%s:%d" % [file, line] if not file.is_empty() else function
	for bt: ScriptBacktrace in script_backtraces:
		if bt != null and bt.get_frame_count() > 0:
			where = "%s:%d" % [bt.get_frame_file(0), bt.get_frame_line(0)]
			break
	var level: String = LEVELS[error_type] if error_type >= 0 and error_type < LEVELS.size() else "error"
	_mutex.lock()
	_queue.append({"level": level, "msg": msg.left(MSG_MAX), "where": where})
	_mutex.unlock()

func _log_message(_message: String, _error: bool) -> void:
	pass

# The queued entries (oldest first); the queue is emptied.
func drain() -> Array[Dictionary]:
	_mutex.lock()
	var out: Array[Dictionary] = _queue.duplicate()
	_queue.clear()
	_mutex.unlock()
	return out
