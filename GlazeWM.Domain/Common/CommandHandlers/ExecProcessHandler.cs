using System;
using System.IO;
using System.Diagnostics;
using System.Windows.Forms;
using GlazeWM.Domain.Common.Commands;
using GlazeWM.Infrastructure.Bussing;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Common.CommandHandlers
{
  public class ExecProcessHandler : ICommandHandler<ExecProcessCommand>
  {
    public CommandResponse Handle(ExecProcessCommand command)
    {
      var processName = command.ProcessName;
      var args = command.Args;
      var shouldElevated = command.ShouldElevated;

      try
      {
        if (shouldElevated)
        {
          using var process = new Process();
          process.StartInfo = new ProcessStartInfo
          {
            // Expand env variables in the process name (eg. "%ProgramFiles%").
            FileName = Environment.ExpandEnvironmentVariables(processName),
            Arguments = string.Join(" ", args),
            UseShellExecute = true,
            Verb = "runas",
            // Set user profile directory as the working dir. This affects the starting directory
            // of terminal processes (eg. CMD, Git bash, etc).
            WorkingDirectory = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile),
          };
          process.Start();
        }
        else
        {
          // a little trick to exec without elevation if GlazeWM is in Admin mode
          // create a shortcut of the process and open it with explorer.
          var fileName = Environment.ExpandEnvironmentVariables(processName);
          var arguments = string.Join(" ", args);

          var profilePath = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
          var savePath = Path.Combine(profilePath, "temp.lnk");

          CreateShortcut(fileName, arguments, savePath);

          var explorerProcess = new Process
          {
            StartInfo = new ProcessStartInfo
            {
              FileName = "explorer.exe",
              Arguments = savePath,
              UseShellExecute = true,
            }
          };
          explorerProcess.Start();
          // delete the shortcut after it's been opened

          // no bug during tests, but it slows down the open speed
          // perhaps there's a better way to handle the potential race condition?
          explorerProcess.WaitForExit();
          File.Delete(savePath);
        }
      }
      catch (Exception exception)
      {
        // Alert the user of the error.
        // TODO: Link to documentation for `exec` command (no proper documentation yet).
        MessageBox.Show(exception.Message);
      }

      return CommandResponse.Ok;
    }
  }
}
