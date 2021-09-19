counter = 1u64
start:
if counter > 0u64 goto print_expression
halt

print_expression:
counter = counter << 1
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