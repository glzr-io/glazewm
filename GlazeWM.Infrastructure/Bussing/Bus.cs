using GlazeWM.Infrastructure.Common.Commands;
using Microsoft.Extensions.Logging;
using System;
using System.Collections.Generic;
using System.Reactive.Subjects;
using System.Threading.Tasks;

namespace GlazeWM.Infrastructure.Bussing
{
  /// <summary>
  /// Bus facilitates communication to command and event handlers.
  /// </summary>
  public sealed class Bus
  {
    public readonly Subject<Event> Events = new();
    public readonly object LockObj = new();
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
      lock (LockObj)
      {
        _logger.LogDebug("Command {commandName} invoked.", command.Name);

        // Create a `Type` object representing the constructed `ICommandHandler` generic.
        var handlerType = typeof(ICommandHandler<>).MakeGenericType(command.GetType());

        var handlerInstance = ServiceLocator.GetRequiredService(handlerType)
          as ICommandHandler<T>;

        return handlerInstance.Handle(command);
      }
    }

    public Task<CommandResponse> InvokeAsync<T>(T command) where T : Command
    {
      return Task.Run(() =>
      {
        try
        {
          return Invoke(command);
        }
        catch (Exception e)
        {
          Invoke(new HandleFatalExceptionCommand(e));
          throw;
        }
      });
    }

    /// <summary>
    /// Sends event to appropriate event handlers.
    /// </summary>
    public void Emit<T>(T @event) where T : Event
    {
      lock (LockObj)
      {
        // Create a `Type` object representing the constructed `IEventHandler` generic.
        var handlerType = typeof(IEventHandler<>).MakeGenericType(@event.GetType());

        var handlerInstances = ServiceLocator.GetServices(handlerType)
          as IEnumerable<IEventHandler<T>>;

        foreach (var handler in handlerInstances)
        {
          handler.Handle(@event);
        }

        // Emit event through subject.
        Events.OnNext(@event);
      }
    }

    public Task EmitAsync<T>(T @event) where T : Event
    {
      return Task.Run(() =>
      {
        try
        {
          Emit(@event);
        }
        catch (Exception e)
        {
          Invoke(new HandleFatalExceptionCommand(e));
        }
      });
    }
  }
}
