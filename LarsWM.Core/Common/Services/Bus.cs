using LarsWM.Core.Common.Models;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace LarsWM.Core.Common.Services
{
    /// <summary>
    /// The event bus
    /// </summary>
    public sealed class Bus : IBus
    {
        private static readonly List<Type> RegisteredCommandHandlers = new List<Type>();
        private static readonly List<Type> RegisteredEventHandlers = new List<Type>();

        /// <summary>
        /// Sends command to appropriate command handler.
        /// </summary>
        public void Invoke<T>(T command) where T : Command
        {
            var commandType = typeof(T);

            // TODO: add centralised error handling here?
            // TODO: create CommandResponse interface with State (failure, success) and Data (any return value from command)?
            foreach (var handler in RegisteredCommandHandlers)
            {
                var handlerInterface = handler.GetInterfaces()[0];
                var handlerCommandType = handlerInterface.GetGenericArguments()[0];

                if (handlerCommandType == commandType)
                {
                    ICommandHandler<T> handlerInstance = Program.ServiceProvider.GetService(handler) as ICommandHandler<T>;
                    handlerInstance.Handle(command);
                }
            }
        }

        /// <summary>
        /// Sends event to appropriate event handler.
        /// </summary>
        public void RaiseEvent<T>(T @event) where T : Event
        {
            throw new NotImplementedException();
        }

        public void RegisterCommandHandler<T>()
        {
            RegisteredCommandHandlers.Add(typeof(T));
        }

        public void RegisterEventHandler<T>()
        {
            RegisteredEventHandlers.Add(typeof(T));
        }
    }
}
