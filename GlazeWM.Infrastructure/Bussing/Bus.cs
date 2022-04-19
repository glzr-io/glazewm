using GlazeWM.Infrastructure.WindowsApi.Events;
using Microsoft.Extensions.DependencyInjection;
using System;
using System.Collections.Generic;
using System.Diagnostics;
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
    public readonly Subject<Event> Events = new Subject<Event>();
    private static readonly Object _lockObj = new Object();

    /// <summary>
    /// Sends command to appropriate command handler.
    /// </summary>
    public CommandResponse Invoke<T>(T command) where T : Command
    {
      try
      {
        Debug.WriteLine($"Command {command.Name} invoked.");

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
        throw error;
      }
    }

    /// <summary>
    /// Sends event to appropriate event handlers.
    /// </summary>
    public void RaiseEvent<T>(T @event) where T : Event
    {
      try
      {
        Debug.WriteLine($"Event {@event.Name} emitted.");

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
        throw error;
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
    private void WriteToErrorLog(Exception error)
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
