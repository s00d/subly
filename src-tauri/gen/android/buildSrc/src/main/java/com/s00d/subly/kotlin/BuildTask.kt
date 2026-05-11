import java.io.File
import org.apache.tools.ant.taskdefs.condition.Os
import org.gradle.api.DefaultTask
import org.gradle.api.GradleException
import org.gradle.api.logging.LogLevel
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.TaskAction

open class BuildTask : DefaultTask() {
    @Input
    var rootDirRel: String? = null
    @Input
    var target: String? = null
    @Input
    var release: Boolean? = null

    @TaskAction
    fun assemble() {
        // The Tauri Android template ships `executable = "node"`, which only
        // works when there is a JS shim called `tauri` next to the Cargo
        // manifest (rare) or a global `tauri` binary in $PATH. On a fresh
        // pnpm setup this fails with:
        //   "Cannot find module '<src-tauri>/tauri'"
        // because Node treats the first arg as a script path.
        //
        // We instead resolve the Tauri CLI through the package manager that
        // owns the project's `node_modules`. The preferred manager is read
        // from `TAURI_CLI_PACKAGE_MANAGER` (set by the outer `pnpm tauri ios|
        // android` invocation) and falls back to pnpm > npm > yarn > node.
        val managers = listOfNotNull(
            System.getenv("TAURI_CLI_PACKAGE_MANAGER"),
            "pnpm",
            "npm",
            "yarn",
            "node",
        )

        var lastException: Exception? = null
        for (executable in managers) {
            try {
                runTauriCli(executable)
                return
            } catch (e: Exception) {
                lastException = e
            }

            if (Os.isFamily(Os.FAMILY_WINDOWS)) {
                for (suffix in listOf(".cmd", ".bat", ".exe")) {
                    try {
                        runTauriCli("$executable$suffix")
                        return
                    } catch (e: Exception) {
                        lastException = e
                    }
                }
            }
        }
        throw lastException ?: GradleException("No package manager found to run tauri CLI")
    }

    fun runTauriCli(executable: String) {
        val rootDirRel = rootDirRel ?: throw GradleException("rootDirRel cannot be null")
        val target = target ?: throw GradleException("target cannot be null")
        val release = release ?: throw GradleException("release cannot be null")
        val args = listOf("tauri", "android", "android-studio-script");

        project.exec {
            workingDir(File(project.projectDir, rootDirRel))
            executable(executable)
            args(args)
            if (project.logger.isEnabled(LogLevel.DEBUG)) {
                args("-vv")
            } else if (project.logger.isEnabled(LogLevel.INFO)) {
                args("-v")
            }
            if (release) {
                args("--release")
            }
            args(listOf("--target", target))
        }.assertNormalExitValue()
    }
}