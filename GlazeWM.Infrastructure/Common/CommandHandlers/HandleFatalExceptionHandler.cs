using System;
using System.IO;
using System.Windows.Forms;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Common.Commands;
using GlazeWM.Infrastructure.Exceptions;
using Microsoft.Extensions.Options;

namespace GlazeWM.Infrastructure.Common.CommandHandlers
{
  internal class HandleFatalExceptionHandler : ICommandHandler<HandleFatalExceptionCommand>
  {
    private readonly Bus _bus;
    private readonly IOptions<ExceptionHandlingOptions> _options;

    public HandleFatalExceptionHandler(Bus bus, IOptions<ExceptionHandlingOptions> options)
    {
      _bus = bus;
      _options = options;
    }

    public CommandResponse Handle(HandleFatalExceptionCommand command)
    {
      var exception = command.Exception;
      var dialogMesssage =
        $"Unhandled exception: {exception.Message}\n\n" + "Continue without exiting?";

      // Alert the user of the error.
      var dialogResult = MessageBox.Show(
        dialogMesssage,
        "Unhandled exception",
        MessageBoxButtons.OKCancel,
        MessageBoxIcon.Warning,
        MessageBoxDefaultButton.Button1,
        MessageBoxOptions.DefaultDesktopOnly
      );

      WriteToErrorLog(exception);

      if (dialogResult == DialogResult.Cancel)
        _bus.Invoke(new ExitApplicationCommand(true));

      return CommandResponse.Ok;
    }

    private void WriteToErrorLog(Exception exception)
    {
      var errorLogPath = _options.Value.ErrorLogPath;

      // Create containing directory. Needs to be created before writing to the file.
      Directory.CreateDirectory(Path.GetDirectoryName(errorLogPath));

      File.AppendAllText(
        errorLogPath,
        _options.Value.ErrorLogMessageDelegate(exception)
      );
    }
  }
}
