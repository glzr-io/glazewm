using System;
using System.Windows.Input;

namespace GlazeWM.Bar.Common
{
  public class RelayCommand : ICommand
  {
    private readonly Action _methodToExecute;
    private readonly Func<bool> _canExecutePredicate;

    public event EventHandler CanExecuteChanged
    {
      add => CommandManager.RequerySuggested += value;
      remove => CommandManager.RequerySuggested -= value;
    }

    public RelayCommand(Action methodToExecute)
    {
      _methodToExecute = methodToExecute
        ?? throw new ArgumentNullException(nameof(methodToExecute));
    }

    public RelayCommand(Action methodToExecute, Func<bool> canExecutePredicate)
      : this(methodToExecute)
    {
      _canExecutePredicate = canExecutePredicate
        ?? throw new ArgumentNullException(nameof(canExecutePredicate));
    }

    public bool CanExecute(object parameter)
    {
      return _canExecutePredicate?.Invoke() != false;
    }

    public void Execute(object parameter)
    {
      _methodToExecute();
    }
  }

  public class RelayCommand<T> : ICommand
  {
    private readonly Action<T> _methodToExecute;
    private readonly Predicate<T> _canExecutePredicate;

    public event EventHandler CanExecuteChanged
    {
      add => CommandManager.RequerySuggested += value;
      remove => CommandManager.RequerySuggested -= value;
    }

    public RelayCommand(Action<T> methodToExecute)
    {
      _methodToExecute = methodToExecute
        ?? throw new ArgumentNullException(nameof(methodToExecute));
    }

    public RelayCommand(Action<T> methodToExecute, Predicate<T> canExecutePredicate)
      : this(methodToExecute)
    {
      _canExecutePredicate = canExecutePredicate
        ?? throw new ArgumentNullException(nameof(canExecutePredicate));
    }

    public bool CanExecute(object parameter)
    {
      return _canExecutePredicate?.Invoke((T)parameter) != false;
    }

    public void Execute(object parameter)
    {
      _methodToExecute((T)parameter);
    }
  }
}
