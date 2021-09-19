counter = 0
start:
if counter < 5 goto print_expression
halt

print_expression:
counter = counter + 1
print 2
print '+'
print 3
print 4
print '='
print 12u64+34u64
print ' '
print ':'
println ')'
goto start