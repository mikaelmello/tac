counter = 0 # test
start:
if counter < 5 goto print_expression
halt

print_expression:
counter = counter + 1
print 1
print 2
print '+'
print 3
print 4
print '='
print 12+34
print ' '
call fun
print ' '
print ':'
println ')'
goto start

fun:
print 1337
return