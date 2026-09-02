class InterfaceException implements Exception {
  const InterfaceException(this.code, this.message);

  final String code;
  final String message;

  @override
  String toString() => 'InterfaceException($code, $message)';
}
