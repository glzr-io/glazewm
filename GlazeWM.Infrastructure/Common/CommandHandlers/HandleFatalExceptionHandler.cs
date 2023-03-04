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

      var dialogButtons = exception is FatalUserException
        ? MessageBoxButtons.OK
        : MessageBoxButtons.OKCancel;

      var dialogMesssage = exception is FatalUserException
        ? exception.Message
        : $"Unhandled exception: {exception.Message}\n\nContinue without exiting?";

      // Alert the user of the error.
      var dialogResult = MessageBox.Show(
        dialogMesssage,
        "Encountered a problem",
        dialogButtons,
        MessageBoxIcon.Warning,
        MessageBoxDefaultButton.Button1,
        MessageBoxOptions.DefaultDesktopOnly
      );

      WriteToErrorLog(exception);

      // Always exit on `FatalUserException`.
      if (dialogResult == DialogResult.Cancel || exception is FatalUserException)
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
