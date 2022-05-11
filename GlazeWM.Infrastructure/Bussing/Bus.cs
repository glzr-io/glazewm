using GlazeWM.Infrastructure.Exceptions;
using GlazeWM.Infrastructure.WindowsApi.Events;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;
using System;
using System.Collections.Generic;
using System.IO;
using System.Reactive.Subjects;
using System.Windows.Forms;

namespace GlazeWM.Infrastructure.Bussing
{
  /// <summary>
  /// Bus facilitates communication to command and event handlers.
  /// </summary>
  public sealed class Bus
  {
    public readonly Subject<Event> Events = new();
    private readonly object _lockObj = new();
    private readonly ILogger<Bus> _logger;

    public Bus(ILogger<Bus> logger)
    {
      _logger = logger;
    }

    /// <summary>
    /// Sends command to appropriate command handler.
    /// </summary>
    public CommandResponse Invoke<T>(T command) where T : Command
    {
      try
      {
        _logger.LogDebug("Command {commandName} invoked.", command.Name);

        // Create a `Type` object representing the constructed `ICommandHandler` generic.
        var handlerType = typeof(ICommandHandler<>).MakeGenericType(command.GetType());

        var handlerInstance = ServiceLocator.Provider.GetRequiredService(handlerType)
          as ICommandHandler<T>;

        lock (_lockObj)
        {
          return handlerInstance.Handle(command);
        }
      }
      catch (Exception error)
      {
        HandleError(error);
        throw;
      }
    }

    /// <summary>
    /// Sends event to appropriate event handlers.
    /// </summary>
    public void RaiseEvent<T>(T @event) where T : Event
    {
      try
      {
        // Create a `Type` object representing the constructed `IEventHandler` generic.
        var handlerType = typeof(IEventHandler<>).MakeGenericType(@event.GetType());

        var handlerInstances = ServiceLocator.Provider.GetServices(handlerType)
          as IEnumerable<IEventHandler<T>>;

        foreach (var handler in handlerInstances)
        {
          lock (_lockObj)
          {
            handler.Handle(@event);
          }
        }

        // Emit event through subject.
        Events.OnNext(@event);
      }
      catch (Exception error)
      {
        HandleError(error);
        throw;
      }
    }

    private void HandleError(Exception error)
    {
      // Alert the user of the error.
      if (error is FatalUserException)
        MessageBox.Show(error.Message);

      WriteToErrorLog(error);
      RaiseEvent(new ApplicationExitingEvent());
    }

    // TODO: Move to dedicated logging service.
    private static void WriteToErrorLog(Exception error)
    {
      var errorLogPath = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.UserProfile),
        "./.glaze-wm/errors.log"
      );

      Directory.CreateDirectory(Path.GetDirectoryName(errorLogPath));
      File.AppendAllText(errorLogPath, $"\n\n{DateTime.Now}\n{error.Message + error.StackTrace}");
    }
  }
}
