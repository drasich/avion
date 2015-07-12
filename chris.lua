-- test
x = 0
--print_ob(foo["ob"])
yop = foo["ob"]
--metatable = getmetatable(yop)
--metatable.__index = metatable
--yop.__metatable = metatable
--print("metatable : ", metatable, " \n");
--object.print_ob(yop)
--yop.get_pos(yop)
--x,y,z = object.get_pos(yop)
caca = yop:__to_string()
print("caca : ", caca)
--yop["position"] = 4
--for n,v in pairs(metatable) do print(n,v) end
print("func start \n");
--for n,v in pairs(object) do print(n,v) end
print("func end \n");
io.write("Just doing nothing \n");
print("what is my pos :: ",  x,y,z, "\n");
return x
