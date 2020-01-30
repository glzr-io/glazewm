using System;
using System.Collections.Generic;
using System.Text;

namespace LarsWM.Core.Common.Models
{
    public interface ICommandHandler<in TCommand, out TResult> where TCommand : ICommand<TResult>
    {
        TResult Handle();
    }
}
